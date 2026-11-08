use super::*;
use crate::model::Project;

#[test]
fn records_merge_by_edit_counter() {
    let mut prj = Project::default();
    let page = prj.pages[0].id;
    let a = add_comment(&mut prj, page, p(10.0, 10.0), "anna", "2026-10-23", "Fuse too small");
    let theirs = prj.review.clone();
    let mut mine = theirs.clone();
    // Mine: status changed, plus a new comment.
    mine.comment_mut(a).unwrap().status = Status::Resolved;
    mine.comment_mut(a).unwrap().bump();
    mine.comments.push(Comment { id: 99, rev: 1, text: "new".into(), ..Default::default() });
    // Theirs: a new markup.
    let mut theirs = theirs;
    theirs.markups.push(Markup { id: 50, rev: 1, ..Default::default() });
    let out = merge(&theirs, &mine);
    assert_eq!(out.comment(a).unwrap().status, Status::Resolved);
    assert_eq!(out.comments.len(), 2);
    assert_eq!(out.markups.len(), 1);
    // Equal counters: the server's version wins.
    let mut other = theirs.clone();
    other.comments[0].text = "other".into();
    other.comments[0].bump();
    let mut mine2 = theirs.clone();
    mine2.comments[0].text = "mine".into();
    mine2.comments[0].bump();
    assert_eq!(merge(&other, &mine2).comments[0].text, "other");
}

#[test]
fn deleted_records_are_kept_as_marks() {
    let mut prj = Project::default();
    let a = add_comment(&mut prj, 0, p(0.0, 0.0), "anna", "", "x");
    let theirs = prj.review.clone();
    let c = prj.review.comment_mut(a).unwrap();
    c.deleted = true;
    c.bump();
    let out = merge(&theirs, &prj.review);
    assert!(out.comment(a).is_none());
    assert_eq!(out.live_comments().count(), 0);
    assert_eq!(out.comments.len(), 1);
}

#[test]
fn hit_comments_and_markups() {
    let mut prj = Project::default();
    let page = prj.pages[0].id;
    let c = add_comment(&mut prj, page, p(20.0, 20.0), "anna", "", "x");
    let m = add_markup(&mut prj, Markup { page, kind: MarkupKind::Rect, pen: Pen::Red, a: p(50.0, 50.0), b: p(80.0, 70.0), text: "".into(), ..Default::default() });
    let l = add_markup(&mut prj, Markup { page, kind: MarkupKind::Line, pen: Pen::Green, a: p(100.0, 100.0), b: p(140.0, 100.0), text: "".into(), ..Default::default() });
    assert_eq!(hit(&prj, page, p(21.0, 21.0), 1.0), Some(Hit::Comment(c)));
    assert_eq!(hit(&prj, page, p(80.5, 60.0), 1.0), Some(Hit::Markup(m)));
    // Inside a rectangle is not on it.
    assert_eq!(hit(&prj, page, p(65.0, 60.0), 1.0), None);
    assert_eq!(hit(&prj, page, p(120.0, 100.4), 1.0), Some(Hit::Markup(l)));
    assert_eq!(hit(&prj, page + 1, p(21.0, 21.0), 1.0), None);
    assert_eq!(prj.review.comment_number(c), 1);
}

#[derive(Default)]
struct Count {
    lines: usize,
    polys: usize,
    texts: Vec<String>,
}

impl Draw for Count {
    fn line(&mut self, _: P, _: P, _: f32, _: Ink, _: bool) {
        self.lines += 1;
    }
    fn poly(&mut self, _: &[P], _: bool, _: f32, _: Ink, _: bool) {
        self.polys += 1;
    }
    fn circle(&mut self, _: P, _: f32, _: f32, _: Ink, _: bool) {}
    fn text(&mut self, _: P, text: &str, _: f32, _: Anchor, _: VAnchor, _: Ink) {
        self.texts.push(text.into());
    }
}

#[test]
fn draws_the_review_layer() {
    let mut prj = Project::default();
    let page = prj.pages[0].id;
    add_comment(&mut prj, page, p(20.0, 20.0), "anna", "", "x");
    add_markup(&mut prj, Markup { page, kind: MarkupKind::Cloud, pen: Pen::Red, a: p(50.0, 50.0), b: p(80.0, 70.0), text: "".into(), ..Default::default() });
    add_markup(&mut prj, Markup { page, kind: MarkupKind::Text, pen: Pen::Green, a: p(10.0, 90.0), b: p(10.0, 90.0), text: "as built".into(), ..Default::default() });
    let mut d = Count::default();
    draw(&mut d, &prj, 0, None);
    assert!(d.polys >= 4, "cloud arcs");
    assert_eq!(d.texts, vec!["as built".to_string(), "1".to_string()]);
    assert!(d.lines >= 1);
}

#[test]
fn change_request_exchange() {
    let mut prj = Project::default();
    prj.pages[0].name = "1".into();
    let id = add_request(&mut prj, "Add emergency stop", "Customer", "2026-10-01");
    assert_eq!(prj.review.request(id).unwrap().number, "CR-001");
    assert_eq!(next_request_number(&prj), "CR-002");
    {
        let r = prj.review.requests.iter_mut().find(|r| r.id == id).unwrap();
        r.pages = vec![prj.pages[0].id];
        r.devices = vec!["-S1".into(), "-K1".into()];
        r.description = "Line 1; \"urgent\"".into();
        r.priority = Priority::High;
    }
    let csv = requests_csv(&prj);
    assert!(csv.starts_with("Number;Title;"));
    let json = requests_json(&prj);

    for text in [csv, json] {
        let mut other = Project::default();
        other.pages[0].name = "1".into();
        assert_eq!(import_requests(&mut other, &text).unwrap(), (1, 0));
        let r = other.review.live_requests().next().unwrap();
        assert_eq!(r.title, "Add emergency stop");
        assert_eq!(r.description, "Line 1; \"urgent\"");
        assert_eq!(r.priority, Priority::High);
        assert_eq!(r.devices, vec!["-S1".to_string(), "-K1".to_string()]);
        assert_eq!(r.pages, vec![other.pages[0].id]);
        // Same file again: nothing changes; a changed status updates the request.
        assert_eq!(import_requests(&mut other, &text).unwrap(), (0, 0));
        let changed = text.replace("\"new\"", "\"accepted\"").replace(";new;", ";accepted;");
        assert_eq!(import_requests(&mut other, &changed).unwrap(), (0, 1));
        assert_eq!(other.review.live_requests().next().unwrap().status, RequestStatus::Accepted);
    }
}

#[test]
fn external_csv_with_other_columns() {
    let mut prj = Project::default();
    let text = "number,title,status,extra\nECR-77,Change motor,In work,x\n,ignored,,\n";
    assert_eq!(import_requests(&mut prj, text).unwrap(), (1, 0));
    let r = prj.review.live_requests().next().unwrap();
    assert_eq!((r.number.as_str(), r.status), ("ECR-77", RequestStatus::InWork));
    assert!(import_requests(&mut prj, "a;b\n1;2\n").is_err());
    assert!(import_requests(&mut prj, "[{").is_err());
}

#[test]
fn signed_in_users_write_in_their_own_name() {
    let theirs = Review {
        approvals: vec![Approval { id: 1, rev: 1, user: "chief".into(), approved: false, ..Default::default() }],
        comments: vec![Comment { id: 2, rev: 1, author: "anna".into(), replies: vec![Reply { author: "anna".into(), ..Default::default() }], ..Default::default() }],
        markups: vec![Markup { id: 3, rev: 1, author: "anna".into(), ..Default::default() }],
        ..Default::default()
    };
    let mut mine = theirs.clone();
    // Approving in the name of another user, changing their approval, rewriting authors.
    mine.approvals[0].approved = true;
    mine.approvals[0].rev = 2;
    mine.approvals.push(Approval { id: 9, rev: 1, user: "chief".into(), approved: true, ..Default::default() });
    mine.comments[0].author = "chief".into();
    mine.comments[0].replies[0].author = "chief".into();
    mine.comments[0].replies.push(Reply { author: "chief".into(), text: "ok".into(), ..Default::default() });
    mine.comments.push(Comment { id: 10, rev: 1, author: "chief".into(), ..Default::default() });
    mine.markups[0].author = "chief".into();
    sign(&theirs, &mut mine, "mallory");
    assert_eq!(mine.approvals[0], theirs.approvals[0], "others' approvals stay");
    assert_eq!(mine.approvals[1].user, "mallory");
    assert_eq!(mine.comments[0].author, "anna");
    let replies: Vec<&str> = mine.comments[0].replies.iter().map(|r| r.author.as_str()).collect();
    assert_eq!(replies, ["anna", "mallory"]);
    assert_eq!(mine.comments[1].author, "mallory");
    assert_eq!(mine.markups[0].author, "anna");
    // Own approvals may change.
    let mut own = theirs.clone();
    own.approvals[0].approved = true;
    own.approvals[0].rev = 2;
    sign(&theirs, &mut own, "Chief");
    assert!(own.approvals[0].approved);
    assert_eq!(own.approvals[0].user, "chief");
}

#[test]
fn replies_keep_their_author_only_unchanged() {
    let reply = |author: &str, text: &str| Reply { author: author.into(), date: "2026-11-08".into(), text: text.into() };
    let theirs = Review { comments: vec![Comment { id: 2, rev: 1, author: "anna".into(), replies: vec![reply("chief", "ok"), reply("anna", "done")], ..Default::default() }], ..Default::default() };
    let mut mine = theirs.clone();
    mine.comments[0].rev = 2;
    // A reply put before the chief's, and the chief's last words rewritten.
    mine.comments[0].replies.insert(0, reply("chief", "Approved, ship it"));
    mine.comments[0].replies[2].text = "not done".into();
    sign(&theirs, &mut mine, "mallory");
    let replies: Vec<(&str, &str)> = mine.comments[0].replies.iter().map(|r| (r.author.as_str(), r.text.as_str())).collect();
    assert_eq!(replies, [("mallory", "Approved, ship it"), ("chief", "ok"), ("mallory", "not done")]);
}

use super::*;

fn titles(sections: &[(&str, Vec<String>)]) -> Vec<String> {
    sections.iter().map(|s| s.0.to_string()).collect()
}

#[test]
fn the_legal_texts_name_the_operator_only_when_there_is_one() {
    // A desktop application has no operator: the disclaimer and the privacy notice.
    let desktop = sections(&LegalNotice::default(), true, Program::Editor);
    assert_eq!(titles(&desktop), [tr("Disclaimer"), tr("Privacy")]);
    assert!(desktop[0].1[0].contains("without warranty"));
    assert!(desktop[1].1.iter().any(|p| p.contains("GitHub for the latest release")));
    assert!(!desktop[1].1.iter().any(|p| p.contains("browser; clearing")));
    // A web editor with an operator: the legal notice first, their privacy text last.
    let notice = LegalNotice {
        name: "ACME GmbH".into(),
        address: "Street 1 | 12345 City".into(),
        email: "info@example.com".into(),
        text: "No dispute resolution.".into(),
        privacy: "Hosted by ACME | Logs are kept for 7 days".into(),
        ..Default::default()
    };
    let web = sections(&notice, false, Program::Editor);
    assert_eq!(titles(&web), [tr("Legal notice"), tr("Disclaimer"), tr("Privacy")]);
    assert_eq!(web[0].1, ["ACME GmbH", "Street 1\n12345 City", "Email: info@example.com", "No dispute resolution."]);
    assert!(web[2].1.iter().any(|p| p.contains("storage of your browser")));
    assert!(!web[2].1.iter().any(|p| p.contains("GitHub for the latest release")));
    assert_eq!(web[2].1[web[2].1.len() - 2..], ["Hosted by ACME", "Logs are kept for 7 days"]);
}

#[test]
fn the_parts_manager_has_a_privacy_notice_of_its_own() {
    for desktop in [true, false] {
        let texts = sections(&LegalNotice::default(), desktop, Program::PartsManager);
        assert_eq!(titles(&texts), [tr("Disclaimer"), tr("Privacy")]);
        let privacy = &texts[1].1;
        assert!(privacy[1].contains("only to the parts server"));
        // It has no projects, no updates and no bug reports to tell about.
        assert!(!privacy.iter().any(|p| p.contains("projects") || p.contains("GitHub")), "{privacy:?}");
        assert!(privacy.iter().any(|p| p.contains("are not stored")));
        assert_eq!(privacy.iter().any(|p| p.contains("IP address")), !desktop);
    }
}

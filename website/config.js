// Settings of the OpenECraft website: where its links lead (docs/website.md). Edit this
// file beside the page after a build; the Docker image writes it from the environment
// variables named at each setting.
window.OPENECRAFT_SITE = {
    // The web editor (OPENECRAFT_EDITOR_URL): a folder beside the page or an address.
    // Empty hides the links to it.
    editor: "editor/",
    // The source repository (OPENECRAFT_REPOSITORY_URL). Empty hides the links to it.
    repository: "https://github.com/nilres/OpenECraft",
    // Downloads (OPENECRAFT_DOWNLOADS_URL); empty: the latest release of the repository.
    downloads: "",
    // The folder of the documentation (OPENECRAFT_DOCUMENTATION_URL), with tutorial.md,
    // plugins.md, …; empty: the docs folder of the repository.
    documentation: "",
    // Where problems are reported (OPENECRAFT_ISSUES_URL); empty: the issues of the
    // repository.
    issues: "",
    // The legal notice (Impressum) of the operator of the site, shown by the link
    // "Legal notice" of the footer. Without a name there is neither link nor notice.
    // Addresses and texts take several lines, separated by "\n" or "|".
    legal: {
        // Person or company running the site (OPENECRAFT_LEGAL_NAME).
        name: "",
        // Postal address (OPENECRAFT_LEGAL_ADDRESS).
        address: "",
        // Email address (OPENECRAFT_LEGAL_EMAIL) and phone number (OPENECRAFT_LEGAL_PHONE).
        email: "",
        phone: "",
        // Who represents a company (OPENECRAFT_LEGAL_REPRESENTATIVE).
        representative: "",
        // Register court and number (OPENECRAFT_LEGAL_REGISTER), VAT ID (OPENECRAFT_LEGAL_VAT_ID).
        register: "",
        vat: "",
        // Responsible for the content, with address (OPENECRAFT_LEGAL_RESPONSIBLE).
        responsible: "",
        // Further text: disclaimers, dispute resolution, … (OPENECRAFT_LEGAL_TEXT).
        text: "",
        // What the operator adds to the privacy notice of the page, which is shown
        // without it too: who hosts the site, what the server logs and for how long,
        // whom to ask (OPENECRAFT_LEGAL_PRIVACY).
        privacy: "",
    },
};

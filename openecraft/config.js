// Settings of a web editor or a web parts manager of OpenECraft: who runs it
// (docs/installation.md). Edit this file beside the page after a build; the Docker image
// of the web editor writes it from the environment variables named at each setting. The
// website has the same settings in its config.js (docs/website.md), which its own web
// editor reads too.
window.OPENECRAFT_SITE = {
    // The legal notice (Impressum) of the operator, shown by Help › Legal and privacy
    // of the editor and by Legal and privacy of the parts manager.
    // Without a name there is none. Addresses and texts take several lines, separated
    // by "\n" or "|".
    legal: {
        // Person or company running the editor (OPENECRAFT_LEGAL_NAME).
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
        // What the operator adds to the privacy notice of the editor: who hosts it,
        // what the server logs and for how long, whom to ask (OPENECRAFT_LEGAL_PRIVACY).
        privacy: "",
    },
};

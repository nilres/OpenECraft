#!/bin/sh
# Writes the settings of the OpenECraft website (website/config.js, docs/website.md) from
# the environment at every start of its Docker image (installer/docker/Dockerfile, target
# website); nginx answers /config.js with the file (website.conf). Each value is written
# as a JavaScript string: quotes and backslashes are escaped, line breaks kept as "\n".
set -eu
mkdir -p /tmp/openecraft
awk '
function js(s,    out, i, c) {
    out = ""
    for (i = 1; i <= length(s); i++) {
        c = substr(s, i, 1)
        if (c == "\\" || c == "\"") out = out "\\" c
        else if (c == "\n") out = out "\\n"
        else if (c != "\r") out = out c
    }
    return "\"" out "\""
}
function setting(indent, name, variable) {
    printf "%s%s: %s,\n", indent, name, js(ENVIRON[variable])
}
BEGIN {
    print "window.OPENECRAFT_SITE = {"
    setting("    ", "editor", "OPENECRAFT_EDITOR_URL")
    setting("    ", "repository", "OPENECRAFT_REPOSITORY_URL")
    setting("    ", "downloads", "OPENECRAFT_DOWNLOADS_URL")
    setting("    ", "documentation", "OPENECRAFT_DOCUMENTATION_URL")
    setting("    ", "issues", "OPENECRAFT_ISSUES_URL")
    print "    legal: {"
    setting("        ", "name", "OPENECRAFT_LEGAL_NAME")
    setting("        ", "address", "OPENECRAFT_LEGAL_ADDRESS")
    setting("        ", "email", "OPENECRAFT_LEGAL_EMAIL")
    setting("        ", "phone", "OPENECRAFT_LEGAL_PHONE")
    setting("        ", "representative", "OPENECRAFT_LEGAL_REPRESENTATIVE")
    setting("        ", "register", "OPENECRAFT_LEGAL_REGISTER")
    setting("        ", "vat", "OPENECRAFT_LEGAL_VAT_ID")
    setting("        ", "responsible", "OPENECRAFT_LEGAL_RESPONSIBLE")
    setting("        ", "text", "OPENECRAFT_LEGAL_TEXT")
    setting("        ", "privacy", "OPENECRAFT_LEGAL_PRIVACY")
    print "    },"
    print "};"
}' > /tmp/openecraft/config.js

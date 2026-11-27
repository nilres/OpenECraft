//! Abbreviations and synonyms of device names for searches: "RCD" or "FI" finds the
//! residual current device, "MCB" or "LS" the circuit breaker. A name gets the keywords
//! of every term it contains; the terms are written in English, German, French and
//! Spanish so that translated names and part descriptions find them too. The keywords
//! are the same in every language (abbreviations are searched as they are written).

/// Terms (lower case, found anywhere in a name) and the keywords they add.
const TERMS: &[(&[&str], &str)] = &[
    (&["residual current", "fehlerstrom", "différentiel", "diferencial"], "RCD RCCB RCBO ELCB GFCI GFI FI DDR ID residual current circuit breaker earth leakage ground fault"),
    (&["circuit breaker", "leitungsschutz", "leistungsschalter", "disjoncteur", "interruptor automático"], "MCB MCCB ACB CB LS breaker miniature circuit breaker"),
    (&["motor protection", "motorschutz", "disjoncteur moteur", "guardamotor"], "MPCB MPS MSS PKZ motor circuit breaker motor starter protector"),
    (&["overload", "überlast", "surcharge", "sobrecarga"], "OL OLR thermal overload relay bimetal"),
    (&["surge", "überspannungsschutz", "parafoudre", "sobretensiones"], "SPD TVSS lightning arrester surge arrester"),
    (&["power supply", "netzteil", "stromversorgung", "alimentation", "fuente de alimentación"], "PSU SMPS power pack 24V DC supply"),
    (&["plc", "sps", "automate", "autómata"], "PLC SPS PLC controller programmable logic controller IO"),
    (&["frequency converter", "frequenzumrichter", "variateur", "variador"], "VFD VSD FU inverter drive variable frequency drive"),
    (&["soft starter", "sanftanlauf", "démarreur progressif", "arrancador suave"], "soft starter SS"),
    (&["contactor", "schütz", "contacteur"], "KM K contactor coil power relay"),
    (&["relay", "relais", "relé"], "K KA relay coil"),
    (&["safety relay", "sicherheitsrelais", "relais de sécurité", "relé de seguridad"], "safety relay e-stop relay monitoring relay"),
    (&["emergency stop", "not-halt", "not-aus", "arrêt d'urgence", "parada de emergencia"], "E-stop ES EMO not-aus not-halt mushroom"),
    (&["disconnector", "switch-disconnector", "trenner", "lasttrenn", "sectionneur", "seccionador"], "isolator isolating switch main switch load break switch"),
    (&["fuse", "sicherung", "fusible"], "F FU fuse cartridge NH LV HBC"),
    (&["transformer", "transformator", "trafo", "transformateur", "transformador"], "T TR trafo transformer"),
    (&["current transformer", "stromwandler", "transformateur de courant", "transformador de corriente"], "CT current transformer"),
    (&["indicator lamp", "pilot light", "pilot lamp", "meldeleuchte", "voyant", "piloto"], "H HL pilot light signal lamp indicator light"),
    (&["push button", "drucktaster", "taster", "bouton-poussoir", "pulsador"], "PB S pushbutton button"),
    (&["selector switch", "wahlschalter", "commutateur", "selector"], "SS selector rotary switch"),
    (&["proximity", "inductive sensor", "capacitive sensor", "näherungs", "détecteur de proximité", "sensor de proximidad"], "prox inductive capacitive BERO initiator proximity switch"),
    (&["photoelectric", "through-beam", "lichtschranke", "lichttaster", "photoélectrique", "fotoeléctrico"], "light barrier light curtain photocell photo eye"),
    (&["limit switch", "position switch", "endschalter", "positionsschalter", "fin de course", "final de carrera"], "LS ZS limit switch end switch"),
    (&["terminal", "klemme", "borne", "borna"], "X XT TB terminal block clamp"),
    (&["motor", "moteur"], "M motor drive"),
    (&["uninterruptible", "unterbrechungsfrei", "onduleur", "ininterrumpida"], "UPS USV battery backup"),
    (&["battery", "batterie", "akku", "batería"], "BAT accumulator cell"),
    (&["inverter", "wechselrichter", "onduleur", "inversor"], "INV PV inverter"),
    (&["solar charge", "laderegler", "régulateur de charge", "regulador de carga"], "MPPT PV charge controller"),
    (&["energy meter", "watt-hour meter", "energiezähler", "stromzähler", "compteur d'énergie", "contador de energía"], "kWh meter electricity meter"),
    (&["hour meter", "betriebsstundenzähler", "compteur horaire", "horómetro"], "hour counter operating hours"),
    (&["thermistor", "ptc", "ntc", "kaltleiter", "heißleiter"], "PTC NTC thermistor temperature sensor"),
    (&["potentiometer", "potentiomètre", "potenciómetro"], "pot poti R"),
    (&["solenoid valve", "magnetventil", "électrovanne", "electroválvula"], "Y YV solenoid valve coil"),
    (&["socket outlet", "steckdose", "prise de courant", "toma de corriente"], "outlet receptacle socket Schuko"),
    (&["protective earth", "schutzleiter", "terre de protection", "tierra de protección"], "PE GND earth ground"),
    (&["light emitting", "led"], "LED light emitting diode"),
];

/// The keywords of a name (part description, symbol name, …), separated by spaces;
/// empty when the name contains none of the terms.
pub fn keywords(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut found: Vec<&str> = Vec::new();
    for (terms, words) in TERMS {
        if terms.iter().any(|t| contains_word(&lower, t)) {
            for w in words.split(' ') {
                if !found.iter().any(|f| f.eq_ignore_ascii_case(w)) {
                    found.push(w);
                }
            }
        }
    }
    found.join(" ")
}

/// Whether `text` contains `term`. Short terms such as "led" or "plc" only count where a
/// word starts, so that "controlled" or "replace" do not match; longer ones also inside
/// a compound word ("Drehstrommotor").
fn contains_word(text: &str, term: &str) -> bool {
    if term.chars().count() > 4 {
        return text.contains(term);
    }
    text.match_indices(term).any(|(i, _)| text[..i].chars().next_back().is_none_or(|c| !c.is_alphanumeric()))
}

#[cfg(test)]
mod tests;

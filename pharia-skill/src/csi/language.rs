use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Deserialize, strum::Display, strum::EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum LanguageCode {
    /// Afrikaans -
    Afr,
    /// Arabic -
    Ara,
    /// Azerbaijani -
    Aze,
    /// Belarusian -
    Bel,
    /// Bengali -
    Ben,
    /// Bosnian -
    Bos,
    /// Bulgarian -
    Bul,
    /// Catalan -
    Cat,
    /// Czech -
    Ces,
    /// Welsh -
    Cym,
    /// Danish -
    Dan,
    /// German -
    Deu,
    /// Greek -
    Ell,
    /// English -
    Eng,
    /// Esperanto -
    Epo,
    /// Estonian -
    Est,
    /// Basque -
    Eus,
    /// Persian -
    Fas,
    /// Finnish -
    Fin,
    /// French -
    Fra,
    /// Irish -
    Gle,
    /// Gujarati -
    Guj,
    /// Hebrew -
    Heb,
    /// Hindi -
    Hin,
    /// Croatian -
    Hrv,
    /// Hungarian -
    Hun,
    /// Armenian -
    Hye,
    /// Indonesian -
    Ind,
    /// Icelandic -
    Isl,
    /// Italian -
    Ita,
    /// Japanese -
    Jpn,
    /// Georgian -
    Kat,
    /// Kazakh -
    Kaz,
    /// Korean -
    Kor,
    /// Latin -
    Lat,
    /// Latvian -
    Lav,
    /// Lithuanian -
    Lit,
    /// Ganda -
    Lug,
    /// Marathi -
    Mar,
    /// Macedonian -
    Mkd,
    /// Mongolian -
    Mon,
    /// Maori -
    Mri,
    /// Malay -
    Msa,
    /// Dutch -
    Nld,
    /// Norwegian Nynorsk -
    Nno,
    /// Norwegian Bokmål -
    Nob,
    /// Punjabi -
    Pan,
    /// Polish -
    Pol,
    /// Portuguese -
    Por,
    /// Romanian -
    Ron,
    /// Russian -
    Rus,
    /// Slovak -
    Slk,
    /// Slovene -
    Slv,
    /// Shona -
    Sna,
    /// Somali -
    Som,
    /// Sotho -
    Sot,
    /// Spanish -
    Spa,
    /// Serbian -
    Srp,
    /// Albanian -
    Sqi,
    /// Swahili -
    Swa,
    /// Swedish -
    Swe,
    /// Tamil -
    Tam,
    /// Telugu -
    Tel,
    /// Tagalog -
    Tgl,
    /// Thai -
    Tha,
    /// Tswana -
    Tsn,
    /// Tsonga -
    Tso,
    /// Turkish -
    Tur,
    /// Ukrainian -
    Ukr,
    /// Urdu -
    Urd,
    /// Vietnamese -
    Vie,
    /// Xhosa -
    Xho,
    /// Yoruba -
    Yor,
    /// Chinese -
    Zho,
    /// Zulu -
    Zul,
}

#[derive(Debug, Serialize)]
pub struct SelectLanguageRequest {
    pub text: String,
    pub languages: Vec<LanguageCode>,
}

impl SelectLanguageRequest {
    pub fn new(text: impl Into<String>, languages: impl Into<Vec<LanguageCode>>) -> Self {
        Self {
            text: text.into(),
            languages: languages.into(),
        }
    }
}

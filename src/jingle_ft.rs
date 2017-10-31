// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use std::collections::BTreeMap;
use std::str::FromStr;

use hashes::Hash;
use jingle::{Creator, ContentId};

use minidom::{Element, IntoElements, IntoAttributeValue, ElementEmitter};
use chrono::{DateTime, FixedOffset};

use error::Error;
use ns;

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub offset: u64,
    pub length: Option<u64>,
    pub hashes: Vec<Hash>,
}

impl IntoElements for Range {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        let mut elem = Element::builder("range")
                               .ns(ns::JINGLE_FT)
                               .attr("offset", if self.offset == 0 { None } else { Some(self.offset) })
                               .attr("length", self.length)
                               .build();
        for hash in self.hashes {
            elem.append_child(hash.into());
        }
        emitter.append_child(elem);
    }
}

type Lang = String;

generate_id!(Desc);

#[derive(Debug, Clone)]
pub struct File {
    pub date: Option<DateTime<FixedOffset>>,
    pub media_type: Option<String>,
    pub name: Option<String>,
    pub descs: BTreeMap<Lang, Desc>,
    pub size: Option<u64>,
    pub range: Option<Range>,
    pub hashes: Vec<Hash>,
}

impl TryFrom<Element> for File {
    type Err = Error;

    fn try_from(elem: Element) -> Result<File, Error> {
        check_self!(elem, "file", ns::JINGLE_FT);
        check_no_attributes!(elem, "file");

        let mut file = File {
            date: None,
            media_type: None,
            name: None,
            descs: BTreeMap::new(),
            size: None,
            range: None,
            hashes: vec!(),
        };

        for child in elem.children() {
            if child.is("date", ns::JINGLE_FT) {
                if file.date.is_some() {
                    return Err(Error::ParseError("File must not have more than one date."));
                }
                file.date = Some(child.text().parse()?);
            } else if child.is("media-type", ns::JINGLE_FT) {
                if file.media_type.is_some() {
                    return Err(Error::ParseError("File must not have more than one media-type."));
                }
                file.media_type = Some(child.text());
            } else if child.is("name", ns::JINGLE_FT) {
                if file.name.is_some() {
                    return Err(Error::ParseError("File must not have more than one name."));
                }
                file.name = Some(child.text());
            } else if child.is("desc", ns::JINGLE_FT) {
                let lang = get_attr!(child, "xml:lang", default);
                let desc = Desc(child.text());
                if file.descs.insert(lang, desc).is_some() {
                    return Err(Error::ParseError("Desc element present twice for the same xml:lang."));
                }
            } else if child.is("size", ns::JINGLE_FT) {
                if file.size.is_some() {
                    return Err(Error::ParseError("File must not have more than one size."));
                }
                file.size = Some(child.text().parse()?);
            } else if child.is("range", ns::JINGLE_FT) {
                if file.range.is_some() {
                    return Err(Error::ParseError("File must not have more than one range."));
                }
                let offset = get_attr!(child, "offset", default);
                let length = get_attr!(child, "length", optional);
                let mut range_hashes = vec!();
                for hash_element in child.children() {
                    if !hash_element.is("hash", ns::HASHES) {
                        return Err(Error::ParseError("Unknown element in JingleFT range."));
                    }
                    range_hashes.push(Hash::try_from(hash_element.clone())?);
                }
                file.range = Some(Range {
                    offset: offset,
                    length: length,
                    hashes: range_hashes,
                });
            } else if child.is("hash", ns::HASHES) {
                file.hashes.push(Hash::try_from(child.clone())?);
            } else {
                return Err(Error::ParseError("Unknown element in JingleFT file."));
            }
        }

        Ok(file)
    }
}

impl From<File> for Element {
    fn from(file: File) -> Element {
        let mut root = Element::builder("file")
                               .ns(ns::JINGLE_FT)
                               .build();
        if let Some(date) = file.date {
            root.append_child(Element::builder("date")
                                      .ns(ns::JINGLE_FT)
                                      .append(date.to_rfc3339())
                                      .build());
        }
        if let Some(media_type) = file.media_type {
            root.append_child(Element::builder("media-type")
                                      .ns(ns::JINGLE_FT)
                                      .append(media_type)
                                      .build());
        }
        if let Some(name) = file.name {
            root.append_child(Element::builder("name")
                                      .ns(ns::JINGLE_FT)
                                      .append(name)
                                      .build());
        }
        for (lang, desc) in file.descs.into_iter() {
            root.append_child(Element::builder("desc")
                                      .ns(ns::JINGLE_FT)
                                      .attr("xml:lang", lang)
                                      .append(desc.0)
                                      .build());
        }
        if let Some(size) = file.size {
            root.append_child(Element::builder("size")
                                      .ns(ns::JINGLE_FT)
                                      .append(format!("{}", size))
                                      .build());
        }
        if let Some(range) = file.range {
            root.append_child(Element::builder("range")
                                      .ns(ns::JINGLE_FT)
                                      .append(range)
                                      .build());
        }
        for hash in file.hashes {
            root.append_child(hash.into());
        }
        root
    }
}
#[derive(Debug, Clone)]
pub struct Description {
    pub file: File,
}

impl TryFrom<Element> for Description {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Description, Error> {
        check_self!(elem, "description", ns::JINGLE_FT, "JingleFT description");
        check_no_attributes!(elem, "JingleFT description");
        let mut file = None;
        for child in elem.children() {
            if file.is_some() {
                return Err(Error::ParseError("JingleFT description element must have exactly one child."));
            }
            file = Some(File::try_from(child.clone())?);
        }
        if file.is_none() {
            return Err(Error::ParseError("JingleFT description element must have exactly one child."));
        }
        Ok(Description {
            file: file.unwrap(),
        })
    }
}

impl From<Description> for Element {
    fn from(description: Description) -> Element {
        let file: Element = description.file.into();
        Element::builder("description")
                .ns(ns::JINGLE_FT)
                .append(file)
                .build()
    }
}

#[derive(Debug, Clone)]
pub struct Checksum {
    pub name: ContentId,
    pub creator: Creator,
    pub file: File,
}

#[derive(Debug, Clone)]
pub struct Received {
    pub name: ContentId,
    pub creator: Creator,
}

impl TryFrom<Element> for Received {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Received, Error> {
        check_self!(elem, "received", ns::JINGLE_FT);
        check_no_children!(elem, "received");
        check_no_unknown_attributes!(elem, "received", ["name", "creator"]);
        Ok(Received {
            name: get_attr!(elem, "name", required),
            creator: get_attr!(elem, "creator", required),
        })
    }
}

impl From<Received> for Element {
    fn from(received: Received) -> Element {
        Element::builder("received")
                .ns(ns::JINGLE_FT)
                .attr("name", received.name)
                .attr("creator", received.creator)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hashes::Algo;
    use base64;

    #[test]
    fn test_description() {
        let elem: Element = r#"
<description xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
  <file>
    <media-type>text/plain</media-type>
    <name>test.txt</name>
    <date>2015-07-26T21:46:00+01:00</date>
    <size>6144</size>
    <hash xmlns='urn:xmpp:hashes:2'
          algo='sha-1'>w0mcJylzCn+AfvuGdqkty2+KP48=</hash>
  </file>
</description>
"#.parse().unwrap();
        let desc = Description::try_from(elem).unwrap();
        assert_eq!(desc.file.media_type, Some(String::from("text/plain")));
        assert_eq!(desc.file.name, Some(String::from("test.txt")));
        assert_eq!(desc.file.descs, BTreeMap::new());
        assert_eq!(desc.file.date, Some(DateTime::parse_from_rfc3339("2015-07-26T21:46:00+01:00").unwrap()));
        assert_eq!(desc.file.size, Some(6144u64));
        assert_eq!(desc.file.range, None);
        assert_eq!(desc.file.hashes[0].algo, Algo::Sha_1);
        assert_eq!(desc.file.hashes[0].hash, base64::decode("w0mcJylzCn+AfvuGdqkty2+KP48=").unwrap());
    }

    #[test]
    fn test_request() {
        let elem: Element = r#"
<description xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
  <file>
    <hash xmlns='urn:xmpp:hashes:2'
          algo='sha-1'>w0mcJylzCn+AfvuGdqkty2+KP48=</hash>
  </file>
</description>
"#.parse().unwrap();
        let desc = Description::try_from(elem).unwrap();
        assert_eq!(desc.file.media_type, None);
        assert_eq!(desc.file.name, None);
        assert_eq!(desc.file.descs, BTreeMap::new());
        assert_eq!(desc.file.date, None);
        assert_eq!(desc.file.size, None);
        assert_eq!(desc.file.range, None);
        assert_eq!(desc.file.hashes[0].algo, Algo::Sha_1);
        assert_eq!(desc.file.hashes[0].hash, base64::decode("w0mcJylzCn+AfvuGdqkty2+KP48=").unwrap());
    }

    #[test]
    fn test_descs() {
        let elem: Element = r#"
<description xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
  <file>
    <media-type>text/plain</media-type>
    <desc xml:lang='fr'>Fichier secret !</desc>
    <desc xml:lang='en'>Secret file!</desc>
    <hash xmlns='urn:xmpp:hashes:2'
          algo='sha-1'>w0mcJylzCn+AfvuGdqkty2+KP48=</hash>
  </file>
</description>
"#.parse().unwrap();
        let desc = Description::try_from(elem).unwrap();
        assert_eq!(desc.file.descs.keys().cloned().collect::<Vec<_>>(), ["en", "fr"]);
        assert_eq!(desc.file.descs["en"], Desc(String::from("Secret file!")));
        assert_eq!(desc.file.descs["fr"], Desc(String::from("Fichier secret !")));

        let elem: Element = r#"
<description xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
  <file>
    <media-type>text/plain</media-type>
    <desc xml:lang='fr'>Fichier secret !</desc>
    <desc xml:lang='fr'>Secret file!</desc>
    <hash xmlns='urn:xmpp:hashes:2'
          algo='sha-1'>w0mcJylzCn+AfvuGdqkty2+KP48=</hash>
  </file>
</description>
"#.parse().unwrap();
        let error = Description::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Desc element present twice for the same xml:lang.");
    }

    #[test]
    fn test_received() {
        let elem: Element = "<received xmlns='urn:xmpp:jingle:apps:file-transfer:5' name='coucou' creator='initiator'/>".parse().unwrap();
        let received = Received::try_from(elem).unwrap();
        assert_eq!(received.name, ContentId(String::from("coucou")));
        assert_eq!(received.creator, Creator::Initiator);
        let elem2 = Element::from(received.clone());
        let received2 = Received::try_from(elem2).unwrap();
        assert_eq!(received2.name, ContentId(String::from("coucou")));
        assert_eq!(received2.creator, Creator::Initiator);

        let elem: Element = "<received xmlns='urn:xmpp:jingle:apps:file-transfer:5' name='coucou' creator='initiator'><coucou/></received>".parse().unwrap();
        let error = Received::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in received element.");

        let elem: Element = "<received xmlns='urn:xmpp:jingle:apps:file-transfer:5' name='coucou' creator='initiator' coucou=''/>".parse().unwrap();
        let error = Received::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in received element.");

        let elem: Element = "<received xmlns='urn:xmpp:jingle:apps:file-transfer:5' creator='initiator'/>".parse().unwrap();
        let error = Received::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'name' missing.");

        let elem: Element = "<received xmlns='urn:xmpp:jingle:apps:file-transfer:5' name='coucou' creator='coucou'/>".parse().unwrap();
        let error = Received::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'creator' attribute.");
    }
}

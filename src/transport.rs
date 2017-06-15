//! Provides transports for the xml streams.

use std::io::prelude::*;

use std::net::{TcpStream, Shutdown};

use xml::reader::{EventReader, XmlEvent as XmlReaderEvent};
use xml::writer::{EventWriter, XmlEvent as XmlWriterEvent, EmitterConfig};

use std::sync::{Arc, Mutex};

use ns;

use minidom;

use locked_io::LockedIO;

use error::Error;

#[allow(unused_imports)]
use openssl::ssl::{SslMethod, Ssl, SslContextBuilder, SslStream, SSL_VERIFY_NONE, SslConnectorBuilder};

use sasl::common::ChannelBinding;

/// A trait which transports are required to implement.
pub trait Transport {
    /// Writes an `xml::writer::XmlEvent` to the stream.
    fn write_event<'a, E: Into<XmlWriterEvent<'a>>>(&mut self, event: E) -> Result<(), Error>;

    /// Reads an `xml::reader::XmlEvent` from the stream.
    fn read_event(&mut self) -> Result<XmlReaderEvent, Error>;

    /// Writes a `minidom::Element` to the stream.
    fn write_element(&mut self, element: &minidom::Element) -> Result<(), Error>;

    /// Reads a `minidom::Element` from the stream.
    fn read_element(&mut self) -> Result<minidom::Element, Error>;

    /// Resets the stream.
    fn reset_stream(&mut self);

    /// Gets channel binding data.
    fn channel_bind(&self) -> ChannelBinding {
        ChannelBinding::None
    }
}

/// A plain text transport, completely unencrypted.
pub struct PlainTransport {
    inner: Arc<Mutex<TcpStream>>, // TODO: this feels rather ugly
    reader: EventReader<LockedIO<TcpStream>>, // TODO: especially feels ugly because
                                              //       this read would keep the lock
                                              //       held very long (potentially)
    writer: EventWriter<LockedIO<TcpStream>>,
}

impl Transport for PlainTransport {
    fn write_event<'a, E: Into<XmlWriterEvent<'a>>>(&mut self, event: E) -> Result<(), Error> {
        Ok(self.writer.write(event)?)
    }

    fn read_event(&mut self) -> Result<XmlReaderEvent, Error> {
        Ok(self.reader.next()?)
    }

    fn write_element(&mut self, element: &minidom::Element) -> Result<(), Error> {
        Ok(element.write_to(&mut self.writer)?)
    }

    fn read_element(&mut self) -> Result<minidom::Element, Error> {
        let element = minidom::Element::from_reader(&mut self.reader)?;
        Ok(element)
    }

    fn reset_stream(&mut self) {
        let locked_io = LockedIO::from(self.inner.clone());
        self.reader = EventReader::new(locked_io.clone());
        self.writer = EventWriter::new_with_config(locked_io, EmitterConfig {
            line_separator: "".into(),
            perform_indent: false,
            normalize_empty_elements: false,
            .. Default::default()
        });
    }

    fn channel_bind(&self) -> ChannelBinding {
        // TODO: channel binding
        ChannelBinding::None
    }
}

impl PlainTransport {
    /// Connects to a server without any encryption.
    pub fn connect(host: &str, port: u16) -> Result<PlainTransport, Error> {
        let tcp_stream = TcpStream::connect((host, port))?;
        let parser = EventReader::new(tcp_stream);
        let parser_stream = parser.into_inner();
        let stream = Arc::new(Mutex::new(parser_stream));
        let locked_io = LockedIO::from(stream.clone());
        let reader = EventReader::new(locked_io.clone());
        let writer = EventWriter::new_with_config(locked_io, EmitterConfig {
            line_separator: "".into(),
            perform_indent: false,
            normalize_empty_elements: false,
            .. Default::default()
        });
        Ok(PlainTransport {
            inner: stream,
            reader: reader,
            writer: writer,
        })
    }

    /// Closes the stream.
    pub fn close(&mut self) {
        self.inner.lock()
                  .unwrap()
                  .shutdown(Shutdown::Both)
                  .unwrap(); // TODO: safety, return value and such
    }
}

/// A transport which uses STARTTLS.
pub struct SslTransport {
    inner: Arc<Mutex<SslStream<TcpStream>>>, // TODO: this feels rather ugly
    reader: EventReader<LockedIO<SslStream<TcpStream>>>, // TODO: especially feels ugly because
                                                         //       this read would keep the lock
                                                         //       held very long (potentially)
    writer: EventWriter<LockedIO<SslStream<TcpStream>>>,
}

impl Transport for SslTransport {
    fn write_event<'a, E: Into<XmlWriterEvent<'a>>>(&mut self, event: E) -> Result<(), Error> {
        Ok(self.writer.write(event)?)
    }

    fn read_event(&mut self) -> Result<XmlReaderEvent, Error> {
        Ok(self.reader.next()?)
    }

    fn write_element(&mut self, element: &minidom::Element) -> Result<(), Error> {
        Ok(element.write_to(&mut self.writer)?)
    }

    fn read_element(&mut self) -> Result<minidom::Element, Error> {
        Ok(minidom::Element::from_reader(&mut self.reader)?)
    }

    fn reset_stream(&mut self) {
        let locked_io = LockedIO::from(self.inner.clone());
        self.reader = EventReader::new(locked_io.clone());
        self.writer = EventWriter::new_with_config(locked_io, EmitterConfig {
            line_separator: "".into(),
            perform_indent: false,
            normalize_empty_elements: false,
            .. Default::default()
        });
    }

    fn channel_bind(&self) -> ChannelBinding {
        // TODO: channel binding
        ChannelBinding::None
    }
}

impl SslTransport {
    /// Connects to a server using STARTTLS.
    pub fn connect(host: &str, port: u16) -> Result<SslTransport, Error> {
        // TODO: very quick and dirty, blame starttls
        let mut stream = TcpStream::connect((host, port))?;
        write!(stream, "<stream:stream xmlns='{}' xmlns:stream='{}' to='{}' version='1.0'>"
                     , ns::CLIENT, ns::STREAM, host)?;
        write!(stream, "<starttls xmlns='{}'/>"
                     , ns::TLS)?;
        let mut parser = EventReader::new(stream);
        loop { // TODO: possibly a timeout?
            match parser.next()? {
                XmlReaderEvent::StartElement { name, .. } => {
                    if let Some(ns) = name.namespace {
                        if ns == ns::TLS && name.local_name == "proceed" {
                            break;
                        }
                        else if ns == ns::STREAM && name.local_name == "error" {
                            return Err(Error::StreamError);
                        }
                    }
                },
                _ => {},
            }
        }
        let stream = parser.into_inner();
        #[cfg(feature = "insecure")]
        let ssl_stream = {
            let mut ctx = SslContextBuilder::new(SslMethod::tls())?;
            ctx.set_verify(SSL_VERIFY_NONE);
            let ssl = Ssl::new(&ctx.build())?;
            ssl.connect(stream)?
        };
        #[cfg(not(feature = "insecure"))]
        let ssl_stream = {
            let ssl_connector = SslConnectorBuilder::new(SslMethod::tls())?.build();
            ssl_connector.connect(host, stream)?
        };
        let ssl_stream = Arc::new(Mutex::new(ssl_stream));
        let locked_io = LockedIO::from(ssl_stream.clone());
        let reader = EventReader::new(locked_io.clone());
        let writer = EventWriter::new_with_config(locked_io, EmitterConfig {
            line_separator: "".into(),
            perform_indent: false,
            normalize_empty_elements: false,
            .. Default::default()
        });
        Ok(SslTransport {
            inner: ssl_stream,
            reader: reader,
            writer: writer,
        })
    }

    /// Closes the stream.
    pub fn close(&mut self) {
        self.inner.lock()
                  .unwrap()
                  .shutdown()
                  .unwrap(); // TODO: safety, return value and such
    }
}
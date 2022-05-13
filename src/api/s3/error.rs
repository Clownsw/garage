use std::convert::TryInto;

use err_derive::Error;
use hyper::header::HeaderValue;
use hyper::{Body, HeaderMap, StatusCode};

use garage_model::helper::error::Error as HelperError;

use crate::common_error::CommonError;
pub use crate::common_error::{CommonErrorDerivative, OkOrBadRequest, OkOrInternalError};
use crate::generic_server::ApiError;
use crate::s3::xml as s3_xml;
use crate::signature::error::Error as SignatureError;

/// Errors of this crate
#[derive(Debug, Error)]
pub enum Error {
	#[error(display = "{}", _0)]
	/// Error from common error
	CommonError(CommonError),

	// Category: cannot process
	/// Authorization Header Malformed
	#[error(display = "Authorization header malformed, expected scope: {}", _0)]
	AuthorizationHeaderMalformed(String),

	/// The object requested don't exists
	#[error(display = "Key not found")]
	NoSuchKey,

	/// The multipart upload requested don't exists
	#[error(display = "Upload not found")]
	NoSuchUpload,

	/// Precondition failed (e.g. x-amz-copy-source-if-match)
	#[error(display = "At least one of the preconditions you specified did not hold")]
	PreconditionFailed,

	/// Parts specified in CMU request do not match parts actually uploaded
	#[error(display = "Parts given to CompleteMultipartUpload do not match uploaded parts")]
	InvalidPart,

	/// Parts given to CompleteMultipartUpload were not in ascending order
	#[error(display = "Parts given to CompleteMultipartUpload were not in ascending order")]
	InvalidPartOrder,

	/// In CompleteMultipartUpload: not enough data
	/// (here we are more lenient than AWS S3)
	#[error(display = "Proposed upload is smaller than the minimum allowed object size")]
	EntityTooSmall,

	// Category: bad request
	/// The request contained an invalid UTF-8 sequence in its path or in other parameters
	#[error(display = "Invalid UTF-8: {}", _0)]
	InvalidUtf8Str(#[error(source)] std::str::Utf8Error),

	/// The request used an invalid path
	#[error(display = "Invalid UTF-8: {}", _0)]
	InvalidUtf8String(#[error(source)] std::string::FromUtf8Error),

	/// The client sent invalid XML data
	#[error(display = "Invalid XML: {}", _0)]
	InvalidXml(String),

	/// The client sent a header with invalid value
	#[error(display = "Invalid header value: {}", _0)]
	InvalidHeader(#[error(source)] hyper::header::ToStrError),

	/// The client sent a range header with invalid value
	#[error(display = "Invalid HTTP range: {:?}", _0)]
	InvalidRange(#[error(from)] (http_range::HttpRangeParseError, u64)),

	/// The client sent a request for an action not supported by garage
	#[error(display = "Unimplemented action: {}", _0)]
	NotImplemented(String),
}

impl<T> From<T> for Error
where
	CommonError: From<T>,
{
	fn from(err: T) -> Self {
		Error::CommonError(CommonError::from(err))
	}
}

impl CommonErrorDerivative for Error {}

impl From<HelperError> for Error {
	fn from(err: HelperError) -> Self {
		match err {
			HelperError::Internal(i) => Self::CommonError(CommonError::InternalError(i)),
			HelperError::BadRequest(b) => Self::CommonError(CommonError::BadRequest(b)),
			HelperError::InvalidBucketName(n) => {
				Self::CommonError(CommonError::InvalidBucketName(n))
			}
			HelperError::NoSuchBucket(n) => Self::CommonError(CommonError::NoSuchBucket(n)),
			e => Self::bad_request(format!("{}", e)),
		}
	}
}

impl From<roxmltree::Error> for Error {
	fn from(err: roxmltree::Error) -> Self {
		Self::InvalidXml(format!("{}", err))
	}
}

impl From<quick_xml::de::DeError> for Error {
	fn from(err: quick_xml::de::DeError) -> Self {
		Self::InvalidXml(format!("{}", err))
	}
}

impl From<SignatureError> for Error {
	fn from(err: SignatureError) -> Self {
		match err {
			SignatureError::CommonError(c) => Self::CommonError(c),
			SignatureError::AuthorizationHeaderMalformed(c) => {
				Self::AuthorizationHeaderMalformed(c)
			}
			SignatureError::InvalidUtf8Str(i) => Self::InvalidUtf8Str(i),
			SignatureError::InvalidHeader(h) => Self::InvalidHeader(h),
		}
	}
}

impl From<multer::Error> for Error {
	fn from(err: multer::Error) -> Self {
		Self::bad_request(err)
	}
}

impl Error {
	pub fn aws_code(&self) -> &'static str {
		match self {
			Error::CommonError(c) => c.aws_code(),
			Error::NoSuchKey => "NoSuchKey",
			Error::NoSuchUpload => "NoSuchUpload",
			Error::PreconditionFailed => "PreconditionFailed",
			Error::InvalidPart => "InvalidPart",
			Error::InvalidPartOrder => "InvalidPartOrder",
			Error::EntityTooSmall => "EntityTooSmall",
			Error::AuthorizationHeaderMalformed(_) => "AuthorizationHeaderMalformed",
			Error::NotImplemented(_) => "NotImplemented",
			Error::InvalidXml(_) => "MalformedXML",
			Error::InvalidRange(_) => "InvalidRange",
			Error::InvalidUtf8Str(_) | Error::InvalidUtf8String(_) | Error::InvalidHeader(_) => {
				"InvalidRequest"
			}
		}
	}
}

impl ApiError for Error {
	/// Get the HTTP status code that best represents the meaning of the error for the client
	fn http_status_code(&self) -> StatusCode {
		match self {
			Error::CommonError(c) => c.http_status_code(),
			Error::NoSuchKey | Error::NoSuchUpload => StatusCode::NOT_FOUND,
			Error::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
			Error::InvalidRange(_) => StatusCode::RANGE_NOT_SATISFIABLE,
			Error::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
			Error::AuthorizationHeaderMalformed(_)
			| Error::InvalidPart
			| Error::InvalidPartOrder
			| Error::EntityTooSmall
			| Error::InvalidXml(_)
			| Error::InvalidUtf8Str(_)
			| Error::InvalidUtf8String(_)
			| Error::InvalidHeader(_) => StatusCode::BAD_REQUEST,
		}
	}

	fn add_http_headers(&self, header_map: &mut HeaderMap<HeaderValue>) {
		use hyper::header;
		#[allow(clippy::single_match)]
		match self {
			Error::InvalidRange((_, len)) => {
				header_map.append(
					header::CONTENT_RANGE,
					format!("bytes */{}", len)
						.try_into()
						.expect("header value only contain ascii"),
				);
			}
			_ => (),
		}
	}

	fn http_body(&self, garage_region: &str, path: &str) -> Body {
		let error = s3_xml::Error {
			code: s3_xml::Value(self.aws_code().to_string()),
			message: s3_xml::Value(format!("{}", self)),
			resource: Some(s3_xml::Value(path.to_string())),
			region: Some(s3_xml::Value(garage_region.to_string())),
		};
		Body::from(s3_xml::to_xml_with_header(&error).unwrap_or_else(|_| {
			r#"
<?xml version="1.0" encoding="UTF-8"?>
<Error>
	<Code>InternalError</Code>
	<Message>XML encoding of error failed</Message>
</Error>
			"#
			.into()
		}))
	}
}

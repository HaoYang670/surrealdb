use crate::iam::Error as IamError;
use crate::idx::ft::MatchRef;
use crate::idx::trees::vector::SharedVector;
use crate::sql::idiom::Idiom;
use crate::sql::index::Distance;
use crate::sql::thing::Thing;
use crate::sql::value::Value;
use crate::sql::TableType;
use crate::syn::error::RenderedError as RenderedParserError;
use crate::vs::Error as VersionstampError;
use base64::DecodeError as Base64Error;
use bincode::Error as BincodeError;
#[cfg(any(
	feature = "kv-mem",
	feature = "kv-surrealkv",
	feature = "kv-rocksdb",
	feature = "kv-fdb",
	feature = "kv-tikv",
))]
use ext_sort::SortError;
use fst::Error as FstError;
use jsonwebtoken::errors::Error as JWTError;
use object_store::Error as ObjectStoreError;
use revision::Error as RevisionError;
use serde::Serialize;
use std::io::Error as IoError;
use std::string::FromUtf8Error;
use storekey::decode::Error as DecodeError;
use storekey::encode::Error as EncodeError;
use thiserror::Error;

/// An error originating from an embedded SurrealDB database.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
	/// This error is used for ignoring a document when processing a query
	#[doc(hidden)]
	#[error("Conditional clause is not truthy")]
	Ignore,

	/// This error is used for breaking a loop in a foreach statement
	#[doc(hidden)]
	#[error("Break statement has been reached")]
	Break,

	/// This error is used for skipping a loop in a foreach statement
	#[doc(hidden)]
	#[error("Continue statement has been reached")]
	Continue,

	/// This error is used for retrying document processing with a new id
	#[doc(hidden)]
	#[error("This document should be retried with a new ID")]
	RetryWithId(Thing),

	/// The database encountered unreachable logic
	#[error("The database encountered unreachable logic: {0}")]
	Unreachable(&'static str),

	/// Statement has been deprecated
	#[error("{0}")]
	Deprecated(String),

	/// A custom error has been thrown
	#[error("An error occurred: {0}")]
	Thrown(String),

	/// There was a problem with the underlying datastore
	#[error("There was a problem with the underlying datastore: {0}")]
	Ds(String),

	/// There was a problem with a datastore transaction
	#[error("There was a problem with a datastore transaction: {0}")]
	Tx(String),

	/// There was an error when starting a new datastore transaction
	#[error("There was an error when starting a new datastore transaction")]
	TxFailure,

	/// The transaction was already cancelled or committed
	#[error("Couldn't update a finished transaction")]
	TxFinished,

	/// The current transaction was created as read-only
	#[error("Couldn't write to a read only transaction")]
	TxReadonly,

	/// The conditional value in the request was not equal
	#[error("Value being checked was not correct")]
	TxConditionNotMet,

	/// The key being inserted in the transaction already exists
	#[error("The key being inserted already exists")]
	TxKeyAlreadyExists,

	/// The key exceeds a limit set by the KV store
	#[error("Record id or key is too large")]
	TxKeyTooLarge,

	/// The value exceeds a limit set by the KV store
	#[error("Record or value is too large")]
	TxValueTooLarge,

	/// The transaction writes too much data for the KV store
	#[error("Transaction is too large")]
	TxTooLarge,

	/// No namespace has been selected
	#[error("Specify a namespace to use")]
	NsEmpty,

	/// No database has been selected
	#[error("Specify a database to use")]
	DbEmpty,

	/// No SQL query has been specified
	#[error("Specify some SQL code to execute")]
	QueryEmpty,

	/// There was an error with the SQL query
	#[error("The SQL query was not parsed fully")]
	QueryRemaining,

	/// There was an error with the SQL query
	#[error("Parse error: {0}")]
	InvalidQuery(RenderedParserError),

	/// There was an error with the SQL query
	#[error("Can not use {value} in a CONTENT clause")]
	InvalidContent {
		value: Value,
	},

	/// There was an error with the SQL query
	#[error("Can not use {value} in a MERGE clause")]
	InvalidMerge {
		value: Value,
	},

	/// There was an error with the provided JSON Patch
	#[error("The JSON Patch contains invalid operations. {message}")]
	InvalidPatch {
		message: String,
	},

	/// Given test operation failed for JSON Patch
	#[error("Given test operation failed for JSON Patch. Expected `{expected}`, but got `{got}` instead.")]
	PatchTest {
		expected: String,
		got: String,
	},

	/// Remote HTTP request functions are not enabled
	#[error("Remote HTTP request functions are not enabled")]
	HttpDisabled,

	/// it is not possible to set a variable with the specified name
	#[error("'{name}' is a protected variable and cannot be set")]
	InvalidParam {
		name: String,
	},

	#[error("Found '{field}' in SELECT clause on line {line}, but field is not an aggregate function, and is not present in GROUP BY expression")]
	InvalidField {
		line: usize,
		field: String,
	},

	/// The FETCH clause accepts idioms, strings and fields.
	#[error("Found {value} on FETCH CLAUSE, but FETCH expects an idiom, a string or fields")]
	InvalidFetch {
		value: Value,
	},

	#[error("Found '{field}' in SPLIT ON clause on line {line}, but field is not present in SELECT expression")]
	InvalidSplit {
		line: usize,
		field: String,
	},

	#[error("Found '{field}' in ORDER BY clause on line {line}, but field is not present in SELECT expression")]
	InvalidOrder {
		line: usize,
		field: String,
	},

	#[error("Found '{field}' in GROUP BY clause on line {line}, but field is not present in SELECT expression")]
	InvalidGroup {
		line: usize,
		field: String,
	},

	/// The LIMIT clause must evaluate to a positive integer
	#[error("Found {value} but the LIMIT clause must evaluate to a positive integer")]
	InvalidLimit {
		value: String,
	},

	/// The START clause must evaluate to a positive integer
	#[error("Found {value} but the START clause must evaluate to a positive integer")]
	InvalidStart {
		value: String,
	},

	/// There was an error with the provided JavaScript code
	#[error("Problem with embedded script function. {message}")]
	InvalidScript {
		message: String,
	},

	/// There was an error with the provided machine learning model
	#[error("Problem with machine learning computation. {message}")]
	InvalidModel {
		message: String,
	},

	/// There was a problem running the specified function
	#[error("There was a problem running the {name}() function. {message}")]
	InvalidFunction {
		name: String,
		message: String,
	},

	/// The wrong quantity or magnitude of arguments was given for the specified function
	#[error("Incorrect arguments for function {name}(). {message}")]
	InvalidArguments {
		name: String,
		message: String,
	},

	/// The wrong quantity or magnitude of arguments was given for the specified function
	#[error("There was a problem running the {name} function. Expected this function to return a value of type {check}, but found {value}")]
	FunctionCheck {
		name: String,
		value: String,
		check: String,
	},

	/// The URL is invalid
	#[error("The URL `{0}` is invalid")]
	InvalidUrl(String),

	/// The size of the vector is incorrect
	#[error("Incorrect vector dimension ({current}). Expected a vector of {expected} dimension.")]
	InvalidVectorDimension {
		current: usize,
		expected: usize,
	},

	/// The size of the vector is incorrect
	#[error("Unable to compute distance.The calculated result is not a valid number: {dist}. Vectors: {left:?} - {right:?}")]
	InvalidVectorDistance {
		left: SharedVector,
		right: SharedVector,
		dist: f64,
	},

	/// The size of the vector is incorrect
	#[error("The vector element ({current}) is not a number.")]
	InvalidVectorType {
		current: String,
		expected: &'static str,
	},

	/// The size of the vector is incorrect
	#[error("The value cannot be converted to a vector: {0}")]
	InvalidVectorValue(String),

	/// Invalid regular expression
	#[error("Invalid regular expression: {0:?}")]
	InvalidRegex(String),

	/// Invalid timeout
	#[error("Invalid timeout: {0:?} seconds")]
	InvalidTimeout(u64),

	/// The query timedout
	#[error("The query was not executed because it exceeded the timeout")]
	QueryTimedout,

	/// The query did not execute, because the transaction was cancelled
	#[error("The query was not executed due to a cancelled transaction")]
	QueryCancelled,

	/// The query did not execute, because the transaction has failed
	#[error("The query was not executed due to a failed transaction")]
	QueryNotExecuted,

	/// The query did not execute, because the transaction has failed (with a message)
	#[error("The query was not executed due to a failed transaction. {message}")]
	QueryNotExecutedDetail {
		message: String,
	},

	/// The permissions do not allow for changing to the specified namespace
	#[error("You don't have permission to change to the {ns} namespace")]
	NsNotAllowed {
		ns: String,
	},

	/// The permissions do not allow for changing to the specified database
	#[error("You don't have permission to change to the {db} database")]
	DbNotAllowed {
		db: String,
	},

	/// The requested namespace does not exist
	#[error("The namespace '{value}' does not exist")]
	NsNotFound {
		value: String,
	},

	/// The requested namespace login does not exist
	#[error("The namespace login '{value}' does not exist")]
	NlNotFound {
		value: String,
	},

	/// The requested database does not exist
	#[error("The database '{value}' does not exist")]
	DbNotFound {
		value: String,
	},

	/// The requested database login does not exist
	#[error("The database login '{value}' does not exist")]
	DlNotFound {
		value: String,
	},

	/// The requested event does not exist
	#[error("The event '{value}' does not exist")]
	EvNotFound {
		value: String,
	},

	/// The requested function does not exist
	#[error("The function 'fn::{value}' does not exist")]
	FcNotFound {
		value: String,
	},

	/// The requested field does not exist
	#[error("The field '{value}' does not exist")]
	FdNotFound {
		value: String,
	},

	/// The requested model does not exist
	#[error("The model 'ml::{value}' does not exist")]
	MlNotFound {
		value: String,
	},

	// The cluster node already exists
	#[error("The node '{value}' already exists")]
	ClAlreadyExists {
		value: String,
	},

	// The cluster node does not exist
	#[error("The node '{value}' does not exist")]
	NdNotFound {
		value: String,
	},

	/// The requested param does not exist
	#[error("The param '${value}' does not exist")]
	PaNotFound {
		value: String,
	},

	/// The requested table does not exist
	#[error("The table '{value}' does not exist")]
	TbNotFound {
		value: String,
	},

	/// The requested live query does not exist
	#[error("The live query '{value}' does not exist")]
	LvNotFound {
		value: String,
	},

	/// The requested cluster live query does not exist
	#[error("The cluster live query '{value}' does not exist")]
	LqNotFound {
		value: String,
	},

	/// The requested analyzer does not exist
	#[error("The analyzer '{value}' does not exist")]
	AzNotFound {
		value: String,
	},

	/// The requested analyzer does not exist
	#[error("The index '{value}' does not exist")]
	IxNotFound {
		value: String,
	},

	/// The requested record does not exist
	#[error("The record '{value}' does not exist")]
	IdNotFound {
		value: String,
	},

	#[error("Unsupported distance: {0}")]
	UnsupportedDistance(Distance),

	/// The requested root user does not exist
	#[error("The root user '{value}' does not exist")]
	UserRootNotFound {
		value: String,
	},

	/// The requested namespace user does not exist
	#[error("The user '{value}' does not exist in the namespace '{ns}'")]
	UserNsNotFound {
		value: String,
		ns: String,
	},

	/// The requested database user does not exist
	#[error("The user '{value}' does not exist in the database '{db}'")]
	UserDbNotFound {
		value: String,
		ns: String,
		db: String,
	},

	/// Unable to perform the realtime query
	#[error("Unable to perform the realtime query")]
	RealtimeDisabled,

	/// Reached excessive computation depth due to functions, subqueries, or futures
	#[error("Reached excessive computation depth due to functions, subqueries, or futures")]
	ComputationDepthExceeded,

	/// Can not execute statement using the specified value
	#[error("Can not execute statement using value '{value}'")]
	InvalidStatementTarget {
		value: String,
	},

	/// Can not execute CREATE statement using the specified value
	#[error("Can not execute CREATE statement using value '{value}'")]
	CreateStatement {
		value: String,
	},

	/// Can not execute UPSERT statement using the specified value
	#[error("Can not execute UPSERT statement using value '{value}'")]
	UpsertStatement {
		value: String,
	},

	/// Can not execute UPDATE statement using the specified value
	#[error("Can not execute UPDATE statement using value '{value}'")]
	UpdateStatement {
		value: String,
	},

	/// Can not execute RELATE statement using the specified value
	#[error("Can not execute RELATE statement using value '{value}'")]
	RelateStatement {
		value: String,
	},

	/// Can not execute RELATE statement using the specified value
	#[error("Can not execute RELATE statement where property 'in' is '{value}'")]
	RelateStatementIn {
		value: String,
	},

	/// Can not execute RELATE statement using the specified value
	#[error("Can not execute RELATE statement where property 'id' is '{value}'")]
	RelateStatementId {
		value: String,
	},

	/// Can not execute RELATE statement using the specified value
	#[error("Can not execute RELATE statement where property 'out' is '{value}'")]
	RelateStatementOut {
		value: String,
	},

	/// Can not execute DELETE statement using the specified value
	#[error("Can not execute DELETE statement using value '{value}'")]
	DeleteStatement {
		value: String,
	},

	/// Can not execute INSERT statement using the specified value
	#[error("Can not execute INSERT statement using value '{value}'")]
	InsertStatement {
		value: String,
	},

	/// Can not execute INSERT statement using the specified value
	#[error("Can not execute INSERT statement where property 'in' is '{value}'")]
	InsertStatementIn {
		value: String,
	},

	/// Can not execute INSERT statement using the specified value
	#[error("Can not execute INSERT statement where property 'id' is '{value}'")]
	InsertStatementId {
		value: String,
	},

	/// Can not execute INSERT statement using the specified value
	#[error("Can not execute INSERT statement where property 'out' is '{value}'")]
	InsertStatementOut {
		value: String,
	},

	/// Can not execute LIVE statement using the specified value
	#[error("Can not execute LIVE statement using value '{value}'")]
	LiveStatement {
		value: String,
	},

	/// Can not execute KILL statement using the specified id
	#[error("Can not execute KILL statement using id '{value}'")]
	KillStatement {
		value: String,
	},

	/// Can not execute CREATE statement using the specified value
	#[error("Expected a single result output when using the ONLY keyword")]
	SingleOnlyOutput,

	/// The permissions do not allow this query to be run on this table
	#[error("You don't have permission to run this query on the `{table}` table")]
	TablePermissions {
		table: String,
	},

	/// The permissions do not allow this query to be run on this table
	#[error("You don't have permission to view the ${name} parameter")]
	ParamPermissions {
		name: String,
	},

	/// The permissions do not allow this query to be run on this table
	#[error("You don't have permission to run the fn::{name} function")]
	FunctionPermissions {
		name: String,
	},

	/// The specified table can not be written as it is setup as a foreign table view
	#[error("Unable to write to the `{table}` table while setup as a view")]
	TableIsView {
		table: String,
	},

	/// A database entry for the specified record already exists
	#[error("Database record `{thing}` already exists")]
	RecordExists {
		thing: String,
	},

	/// A database index entry for the specified record already exists
	#[error("Database index `{index}` already contains {value}, with record `{thing}`")]
	IndexExists {
		thing: Thing,
		index: String,
		value: String,
	},

	/// The specified table is not configured for the type of record being added
	#[error("Found record: `{thing}` which is {}a relation, but expected a {target_type}", if *relation { "not " } else { "" })]
	TableCheck {
		thing: String,
		relation: bool,
		target_type: TableType,
	},

	/// The specified field did not conform to the field type check
	#[error("Found {value} for field `{field}`, with record `{thing}`, but expected a {check}")]
	FieldCheck {
		thing: String,
		value: String,
		field: Idiom,
		check: String,
	},

	/// The specified field did not conform to the field ASSERT clause
	#[error("Found {value} for field `{field}`, with record `{thing}`, but field must conform to: {check}")]
	FieldValue {
		thing: String,
		value: String,
		field: Idiom,
		check: String,
	},

	/// The specified value did not conform to the LET type check
	#[error("Found {value} for param ${name}, but expected a {check}")]
	SetCheck {
		value: String,
		name: String,
		check: String,
	},

	/// The specified field did not conform to the field ASSERT clause
	#[error(
		"Found changed value for field `{field}`, with record `{thing}`, but field is readonly"
	)]
	FieldReadonly {
		thing: String,
		field: Idiom,
	},

	/// Found a record id for the record but we are creating a specific record
	#[error("Found {value} for the id field, but a specific record has been specified")]
	IdMismatch {
		value: String,
	},

	/// Found a record id for the record but this is not a valid id
	#[error("Found {value} for the Record ID but this is not a valid id")]
	IdInvalid {
		value: String,
	},

	/// Unable to coerce to a value to another value
	#[error("Expected a {into} but found {from}")]
	CoerceTo {
		from: Value,
		into: String,
	},

	/// Unable to convert a value to another value
	#[error("Expected a {into} but cannot convert {from} into a {into}")]
	ConvertTo {
		from: Value,
		into: String,
	},

	/// Unable to coerce to a value to another value
	#[error("Expected a {kind} but the array had {size} items")]
	LengthInvalid {
		kind: String,
		size: usize,
	},

	/// Cannot perform addition
	#[error("Cannot perform addition with '{0}' and '{1}'")]
	TryAdd(String, String),

	/// Cannot perform subtraction
	#[error("Cannot perform subtraction with '{0}' and '{1}'")]
	TrySub(String, String),

	/// Cannot perform multiplication
	#[error("Cannot perform multiplication with '{0}' and '{1}'")]
	TryMul(String, String),

	/// Cannot perform division
	#[error("Cannot perform division with '{0}' and '{1}'")]
	TryDiv(String, String),

	/// Cannot perform remainder
	#[error("Cannot perform remainder with '{0}' and '{1}'")]
	TryRem(String, String),

	/// Cannot perform power
	#[error("Cannot raise the value '{0}' with '{1}'")]
	TryPow(String, String),

	/// Cannot perform negation
	#[error("Cannot negate the value '{0}'")]
	TryNeg(String),

	/// It's is not possible to convert between the two types
	#[error("Cannot convert from '{0}' to '{1}'")]
	TryFrom(String, &'static str),

	/// There was an error processing a remote HTTP request
	#[error("There was an error processing a remote HTTP request: {0}")]
	Http(String),

	/// There was an error processing a value in parallel
	#[error("There was an error processing a value in parallel: {0}")]
	Channel(String),

	/// Represents an underlying error with IO encoding / decoding
	#[error("I/O error: {0}")]
	Io(#[from] IoError),

	/// Represents an error when encoding a key-value entry
	#[error("Key encoding error: {0}")]
	Encode(#[from] EncodeError),

	/// Represents an error when decoding a key-value entry
	#[error("Key decoding error: {0}")]
	Decode(#[from] DecodeError),

	/// Represents an underlying error with versioned data encoding / decoding
	#[error("Versioned error: {0}")]
	Revision(#[from] RevisionError),

	/// The index has been found to be inconsistent
	#[error("Index is corrupted: {0}")]
	CorruptedIndex(&'static str),

	/// The query planner did not find an index able to support the match @@ for a given expression
	#[error("There was no suitable index supporting the expression '{value}'")]
	NoIndexFoundForMatch {
		value: String,
	},

	/// Represents an error when analyzing a value
	#[error("A value can't be analyzed: {0}")]
	AnalyzerError(String),

	/// Represents an error when trying to highlight a value
	#[error("A value can't be highlighted: {0}")]
	HighlightError(String),

	/// Represents an underlying error with Bincode serializing / deserializing
	#[error("Bincode error: {0}")]
	Bincode(#[from] BincodeError),

	/// Represents an underlying error with FST
	#[error("FstError error: {0}")]
	FstError(#[from] FstError),

	/// Represents an underlying error while reading UTF8 characters
	#[error("Utf8 error: {0}")]
	Utf8Error(#[from] FromUtf8Error),

	/// Represents an underlying error with the Object Store
	#[error("Object Store error: {0}")]
	ObsError(#[from] ObjectStoreError),

	/// There was an error with model computation
	#[error("There was an error with model computation: {0}")]
	ModelComputation(String),

	/// The feature has not yet being implemented
	#[error("Feature not yet implemented: {feature}")]
	FeatureNotYetImplemented {
		feature: String,
	},

	/// Duplicated match references are not allowed
	#[error("Duplicated Match reference: {mr}")]
	DuplicatedMatchRef {
		mr: MatchRef,
	},

	/// Represents a failure in timestamp arithmetic related to database internals
	#[error("Timestamp arithmetic error: {0}")]
	TimestampOverflow(String),

	/// Internal server error
	/// This should be used extremely sporadically, since we lose the type of error as a consequence
	/// There will be times when it is useful, such as with unusual type conversion errors
	#[error("Internal database error: {0}")]
	Internal(String),

	/// Unimplemented functionality
	#[error("Unimplemented functionality: {0}")]
	Unimplemented(String),

	#[error("Versionstamp in key is corrupted: {0}")]
	CorruptedVersionstampInKey(#[from] VersionstampError),

	/// Invalid level
	#[error("Invalid level '{0}'")]
	InvalidLevel(String),

	/// Represents an underlying IAM error
	#[error("IAM error: {0}")]
	IamError(#[from] IamError),

	//
	// Capabilities
	//
	/// Scripting is not allowed
	#[error("Scripting functions are not allowed")]
	ScriptingNotAllowed,

	/// Function is not allowed
	#[error("Function '{0}' is not allowed to be executed")]
	FunctionNotAllowed(String),

	/// Network target is not allowed
	#[error("Access to network target '{0}' is not allowed")]
	NetTargetNotAllowed(String),

	//
	// Authentication / Signup
	//
	#[error("There was an error creating the token")]
	TokenMakingFailed,

	#[error("No record was returned")]
	NoRecordFound,

	#[error("The signup query failed")]
	SignupQueryFailed,

	#[error("The signin query failed")]
	SigninQueryFailed,

	#[error("Username or Password was not provided")]
	MissingUserOrPass,

	#[error("No signin target to either SC or DB or NS or KV")]
	NoSigninTarget,

	#[error("The password did not verify")]
	InvalidPass,

	/// There was an error with authentication
	#[error("There was a problem with authentication")]
	InvalidAuth,

	/// There was an error with signing up
	#[error("There was a problem with signing up")]
	InvalidSignup,

	/// Auth was expected to be set but was unknown
	#[error("Auth was expected to be set but was unknown")]
	UnknownAuth,

	/// Auth requires a token header which is missing
	#[error("Auth token is missing the '{0}' header")]
	MissingTokenHeader(String),

	/// Auth requires a token claim which is missing
	#[error("Auth token is missing the '{0}' claim")]
	MissingTokenClaim(String),

	/// The db is running without an available storage engine
	#[error("The db is running without an available storage engine")]
	MissingStorageEngine,

	/// The requested analyzer already exists
	#[error("The analyzer '{value}' already exists")]
	AzAlreadyExists {
		value: String,
	},

	/// The requested database already exists
	#[error("The database '{value}' already exists")]
	DbAlreadyExists {
		value: String,
	},

	/// The requested event already exists
	#[error("The event '{value}' already exists")]
	EvAlreadyExists {
		value: String,
	},

	/// The requested field already exists
	#[error("The field '{value}' already exists")]
	FdAlreadyExists {
		value: String,
	},

	/// The requested function already exists
	#[error("The function 'fn::{value}' already exists")]
	FcAlreadyExists {
		value: String,
	},

	/// The requested index already exists
	#[error("The index '{value}' already exists")]
	IxAlreadyExists {
		value: String,
	},

	/// The requested model already exists
	#[error("The model '{value}' already exists")]
	MlAlreadyExists {
		value: String,
	},

	/// The requested namespace already exists
	#[error("The namespace '{value}' already exists")]
	NsAlreadyExists {
		value: String,
	},

	/// The requested param already exists
	#[error("The param '${value}' already exists")]
	PaAlreadyExists {
		value: String,
	},

	/// The requested table already exists
	#[error("The table '{value}' already exists")]
	TbAlreadyExists {
		value: String,
	},

	/// The requested namespace token already exists
	#[error("The namespace token '{value}' already exists")]
	NtAlreadyExists {
		value: String,
	},

	/// The requested database token already exists
	#[error("The database token '{value}' already exists")]
	DtAlreadyExists {
		value: String,
	},

	/// The requested user already exists
	#[error("The root user '{value}' already exists")]
	UserRootAlreadyExists {
		value: String,
	},

	/// The requested namespace user already exists
	#[error("The user '{value}' already exists in the namespace '{ns}'")]
	UserNsAlreadyExists {
		value: String,
		ns: String,
	},

	/// The requested database user already exists
	#[error("The user '{value}' already exists in the database '{db}'")]
	UserDbAlreadyExists {
		value: String,
		ns: String,
		db: String,
	},

	/// The session has expired either because the token used
	/// to establish it has expired or because an expiration
	/// was explicitly defined when establishing it
	#[error("The session has expired")]
	ExpiredSession,

	/// A node task has failed
	#[error("A node task has failed: {0}")]
	NodeAgent(&'static str),

	/// The supplied type could not be serialiazed into `sql::Value`
	#[error("Serialization error: {0}")]
	Serialization(String),

	/// The requested root access method already exists
	#[error("The root access method '{ac}' already exists")]
	AccessRootAlreadyExists {
		ac: String,
	},

	/// The requested namespace access method already exists
	#[error("The access method '{ac}' already exists in the namespace '{ns}'")]
	AccessNsAlreadyExists {
		ac: String,
		ns: String,
	},

	/// The requested database access method already exists
	#[error("The access method '{ac}' already exists in the database '{db}'")]
	AccessDbAlreadyExists {
		ac: String,
		ns: String,
		db: String,
	},

	/// The requested root access method does not exist
	#[error("The root access method '{ac}' does not exist")]
	AccessRootNotFound {
		ac: String,
	},

	/// The requested root access grant does not exist
	#[error("The root access grant '{gr}' does not exist")]
	AccessGrantRootNotFound {
		ac: String,
		gr: String,
	},

	/// The requested namespace access method does not exist
	#[error("The access method '{ac}' does not exist in the namespace '{ns}'")]
	AccessNsNotFound {
		ac: String,
		ns: String,
	},

	/// The requested namespace access grant does not exist
	#[error("The access grant '{gr}' does not exist in the namespace '{ns}'")]
	AccessGrantNsNotFound {
		ac: String,
		gr: String,
		ns: String,
	},

	/// The requested database access method does not exist
	#[error("The access method '{ac}' does not exist in the database '{db}'")]
	AccessDbNotFound {
		ac: String,
		ns: String,
		db: String,
	},

	/// The requested database access grant does not exist
	#[error("The access grant '{gr}' does not exist in the database '{db}'")]
	AccessGrantDbNotFound {
		ac: String,
		gr: String,
		ns: String,
		db: String,
	},

	/// The access method cannot be defined on the requested level
	#[error("The access method cannot be defined on the requested level")]
	AccessLevelMismatch,

	#[error("The access method cannot be used in the requested operation")]
	AccessMethodMismatch,

	#[error("The access method does not exist")]
	AccessNotFound,

	#[error("This access method has an invalid duration")]
	AccessInvalidDuration,

	#[error("This access method results in an invalid expiration")]
	AccessInvalidExpiration,

	#[error("The record access signup query failed")]
	AccessRecordSignupQueryFailed,

	#[error("The record access signin query failed")]
	AccessRecordSigninQueryFailed,

	#[error("This record access method does not allow signup")]
	AccessRecordNoSignup,

	#[error("This record access method does not allow signin")]
	AccessRecordNoSignin,

	#[error("This bearer access method requires a key to be provided")]
	AccessBearerMissingKey,

	#[error("This bearer access grant has an invalid format")]
	AccessGrantBearerInvalid,

	#[error("This access grant has an invalid subject")]
	AccessGrantInvalidSubject,

	#[error("This access grant has been revoked")]
	AccessGrantRevoked,

	/// Found a table name for the record but this is not a valid table
	#[error("Found {value} for the Record ID but this is not a valid table name")]
	TbInvalid {
		value: String,
	},

	/// This error is used for breaking execution when a value is returned
	#[doc(hidden)]
	#[error("Return statement has been reached")]
	Return {
		value: Value,
	},

	/// A destructuring variant was used in a context where it is not supported
	#[error("{variant} destructuring method is not supported here")]
	UnsupportedDestructure {
		variant: String,
	},

	#[doc(hidden)]
	#[error("The underlying datastore does not support versioned queries")]
	UnsupportedVersionedQueries,
}

impl From<Error> for String {
	fn from(e: Error) -> String {
		e.to_string()
	}
}

impl From<Base64Error> for Error {
	fn from(_: Base64Error) -> Error {
		Error::InvalidAuth
	}
}

impl From<JWTError> for Error {
	fn from(_: JWTError) -> Error {
		Error::InvalidAuth
	}
}

impl From<regex::Error> for Error {
	fn from(error: regex::Error) -> Self {
		Error::InvalidRegex(error.to_string())
	}
}

#[cfg(feature = "kv-mem")]
impl From<echodb::err::Error> for Error {
	fn from(e: echodb::err::Error) -> Error {
		match e {
			echodb::err::Error::KeyAlreadyExists => Error::TxKeyAlreadyExists,
			echodb::err::Error::ValNotExpectedValue => Error::TxConditionNotMet,
			_ => Error::Tx(e.to_string()),
		}
	}
}

#[cfg(feature = "kv-indxdb")]
impl From<indxdb::err::Error> for Error {
	fn from(e: indxdb::err::Error) -> Error {
		match e {
			indxdb::err::Error::KeyAlreadyExists => Error::TxKeyAlreadyExists,
			indxdb::err::Error::ValNotExpectedValue => Error::TxConditionNotMet,
			_ => Error::Tx(e.to_string()),
		}
	}
}

#[cfg(feature = "kv-tikv")]
impl From<tikv::Error> for Error {
	fn from(e: tikv::Error) -> Error {
		match e {
			tikv::Error::DuplicateKeyInsertion => Error::TxKeyAlreadyExists,
			tikv::Error::KeyError(ke) if ke.abort.contains("KeyTooLarge") => Error::TxKeyTooLarge,
			tikv::Error::RegionError(re) if re.raft_entry_too_large.is_some() => Error::TxTooLarge,
			_ => Error::Tx(e.to_string()),
		}
	}
}

#[cfg(feature = "kv-rocksdb")]
impl From<rocksdb::Error> for Error {
	fn from(e: rocksdb::Error) -> Error {
		Error::Tx(e.to_string())
	}
}

#[cfg(feature = "kv-surrealkv")]
impl From<surrealkv::Error> for Error {
	fn from(e: surrealkv::Error) -> Error {
		Error::Tx(e.to_string())
	}
}

#[cfg(feature = "kv-fdb")]
impl From<foundationdb::FdbError> for Error {
	fn from(e: foundationdb::FdbError) -> Error {
		Error::Ds(e.to_string())
	}
}

#[cfg(feature = "kv-fdb")]
impl From<foundationdb::TransactionCommitError> for Error {
	fn from(e: foundationdb::TransactionCommitError) -> Error {
		Error::Tx(e.to_string())
	}
}

impl From<channel::RecvError> for Error {
	fn from(e: channel::RecvError) -> Error {
		Error::Channel(e.to_string())
	}
}

impl<T> From<channel::SendError<T>> for Error {
	fn from(e: channel::SendError<T>) -> Error {
		Error::Channel(e.to_string())
	}
}

#[cfg(any(feature = "http", feature = "jwks"))]
impl From<reqwest::Error> for Error {
	fn from(e: reqwest::Error) -> Error {
		Error::Http(e.to_string())
	}
}

#[cfg(any(
	feature = "kv-mem",
	feature = "kv-surrealkv",
	feature = "kv-rocksdb",
	feature = "kv-fdb",
	feature = "kv-tikv",
))]
impl<S, D, I> From<SortError<S, D, I>> for Error
where
	S: std::error::Error,
	D: std::error::Error,
	I: std::error::Error,
{
	fn from(e: SortError<S, D, I>) -> Error {
		Error::Internal(e.to_string())
	}
}

impl Serialize for Error {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(self.to_string().as_str())
	}
}
impl Error {
	pub fn set_check_from_coerce(self, name: String) -> Error {
		match self {
			Error::CoerceTo {
				from,
				into,
			} => Error::SetCheck {
				name,
				value: from.to_string(),
				check: into,
			},
			e => e,
		}
	}

	pub fn function_check_from_coerce(self, name: impl Into<String>) -> Error {
		match self {
			Error::CoerceTo {
				from,
				into,
			} => Error::FunctionCheck {
				name: name.into(),
				value: from.to_string(),
				check: into,
			},
			e => e,
		}
	}
}

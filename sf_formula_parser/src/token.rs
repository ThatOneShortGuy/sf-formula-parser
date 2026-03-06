/// A text string or number you enter that is not calculated or changed.
/// For example, if you have a value that’s always multiplied by 2% of an amount, your formula would contain the literal value of 2% of that amount:
///
/// ```sff
/// ROUND((Amount*0.02), 2)
/// ```
/// This example contains every possible part of a formula:
///
/// - A function called ROUND used to return a number rounded to a specified number of decimal places.
/// - A field reference called Amount.
/// - An operator, *, that tells the formula builder to multiply the contents of the Amount field by the literal value, 0.02.
/// - A literal number, 0.02. Use the decimal value for all percents. To include actual text in your formula, enclose it in quotes.
/// - The last number 2 in this formula is the input required for the ROUND function that determines the number of decimal places to return.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LiteralValue<'s> {
    Checkbox(bool),
    Number(f64),
    Text(&'s str),
}

impl<'s> From<&'s str> for LiteralValue<'s> {
    fn from(value: &'s str) -> Self {
        LiteralValue::Text(value)
    }
}

impl<'s> From<f64> for LiteralValue<'s> {
    fn from(value: f64) -> Self {
        LiteralValue::Number(value)
    }
}

impl<'s> From<isize> for LiteralValue<'s> {
    fn from(value: isize) -> Self {
        LiteralValue::Number(value as f64)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldReference<'s> {
    pub field: &'s str,
    pub next: Option<Box<FieldReference<'s>>>,
}

impl<'s> FieldReference<'s> {
    pub fn new<T: Into<&'s str>>(field: T) -> Self {
        Self {
            field: field.into(),
            next: None,
        }
    }
    pub fn with_next(mut self, next: impl Into<Self>) -> Self {
        if self.next.is_none() {
            self.next = Some(Box::new(next.into()));
        }
        self
    }

    pub fn from_iter<T: IntoIterator<Item = &'s str>>(iter: T) -> Option<Self> {
        let init = None::<FieldReference<'s>>;
        iter.into_iter().fold(init, |acc, s| match acc {
            Some(acc) => Some(acc.with_next(s)),
            None => Some(s.into()),
        })
    }
}

impl<'s, T: Into<&'s str>> From<T> for FieldReference<'s> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FunctionName {
    /// Calculates the absolute value of a number. The absolute value of a number is the number without its positive or negative sign.
    ABS,
    /// Returns the date that is the indicated number of months before or after a specified date. If the specified date is the last day of the month, the resulting date is the last day of the resulting month. Otherwise, the result has the same date component as the specified date.
    ADDMONTHS,
    /// Returns a TRUE response if all values are true; returns a FALSE response if one or more values are false.
    AND,
    /// Determines if text begins with specific characters and returns TRUE if it does. Returns FALSE if it doesn't.
    BEGINS,
    /// Determines if an expression has a value and returns a substitute expression if it doesn’t. If the expression has a value, returns the value of the expression.
    BLANKVALUE,
    /// Inserts a line break in a string of text.
    BR,
    /// Checks a given expression against a series of values. If the expression is equal to a value, returns the corresponding result. If it isn't equal to any of the values, returns the else_result.
    CASE,
    /// Converts a 15-character ID to a case-insensitive 18-character ID. In Salesforce Classic, the function converts only valid Salesforce 15-character IDs. If you pass in an invalid ID, the function returns the ID passed in. In Lightning Experience, the function converts any 15-character ID.
    CASESAFEID,
    /// Rounds a number up to the nearest integer, away from zero if negative.
    CEILING,
    /// Compares two arguments of text and returns TRUE if the first argument contains the second argument. If not, returns FALSE.
    CONTAINS,
    /// Returns the conversion rate to the corporate currency for the given currency ISO code. If the currency is invalid, returns 1.0.
    CURRENCYRATE,
    /// Returns a date value from the year, month, and day values you enter. Salesforce displays an error on the detail page if the value of the DATE function in a formula field is an invalid date, such as February 29 in a non-leap year.
    DATE,
    /// Returns a date value for a date/time or text expression.
    DATEVALUE,
    /// Returns a year, month, day, and GMT time value.
    DATETIMEVALUE,
    /// Returns a day of the month in the form of a number from 1 through 31.
    DAY,
    /// Calculates the distance between two locations in miles or kilometers.
    DISTANCE,
    /// Returns a value for e raised to the power of a number you specify.
    EXP,
    /// Returns the position of a string within a string of text represented as a number.
    FIND,
    /// Returns a number rounded down to the nearest integer, towards zero if negative.
    FLOOR,
    /// Returns a geolocation based on the provided latitude and longitude. Must be used with the DISTANCE function.
    GEOLOCATION,
    /// Returns an array of strings in the form of record IDs for the selected records in a list, such as a list view or related list.
    GETRECORDIDS,
    /// Returns the user’s session ID.
    GETSESSIONID,
    /// Returns the local time hour value without the date in the form of a number from 1 through 24.
    HOUR,
    /// Encodes text and merge field values for use in HTML by replacing characters that are reserved in HTML, such as the greater-than sign (>), with HTML entity equivalents, such as &gt;.
    HTMLENCODE,
    /// Creates a link to a URL specified that is linkable from the text specified.
    HYPERLINK,
    /// Determines if expressions are true or false. Returns a given value if true and another value if false.
    IF,
    /// Inserts an image with alternate text and height and width specifications.
    IMAGE,
    /// Securely retrieves external images and prevents unauthorized requests for user credentials.
    IMAGEPROXYURL,
    /// Returns content from an s-control snippet. Use this function to reuse common code in many s-controls.
    INCLUDE,
    /// Determines if any value selected in a multi-select picklist field equals a text literal you specify.
    INCLUDES,
    /// Determines if an expression has a value and returns TRUE if it does not. If it contains a value, this function returns FALSE.
    ISBLANK,
    /// Compares the value of a field to the previous value and returns TRUE if the values are different. If the values are the same, this function returns FALSE.
    ISCHANGED,
    /// Checks if the record is a clone of another record and returns TRUE if one item is a clone. Otherwise, returns FALSE.
    ISCLONE,
    /// Checks if the formula is running during the creation of a new record and returns TRUE if it is. If an existing record is being updated, this function returns FALSE.
    ISNEW,
    /// Determines if an expression is null (blank) and returns TRUE if it is. If it contains a value, this function returns FALSE.
    ISNULL,
    /// Determines if a text value is a number and returns TRUE if it is. Otherwise, returns FALSE.
    ISNUMBER,
    /// Determines if the value of a picklist field is equal to a text literal you specify.
    ISPICKVAL,
    /// Encodes text and merge field values for use in JavaScript by inserting escape characters, such as a backslash (\), before unsafe JavaScript characters, such as the apostrophe (').
    JSENCODE,
    /// Encodes text and merge field values for use in JavaScript inside HTML tags by replacing characters that are reserved in HTML with HTML entity equivalents and inserting escape characters before unsafe JavaScript characters.
    JSINHTMLENCODE,
    /// Returns a JunctionIDList based on the provided IDs.
    JUNCTIONIDLIST,
    /// Returns the specified number of characters from the beginning of a text string.
    LEFT,
    /// Returns the number of characters in a specified text string.
    LEN,
    /// Returns a relative URL in the form of a link (href and anchor tags) for a custom s-control or Salesforce page.
    LINKTO,
    /// Returns the natural logarithm of a specified number. Natural logarithms are based on the constant e value of 2.71828182845904.
    LN,
    /// Returns the base 10 logarithm of a number.
    LOG,
    /// Converts all letters in the specified text string to lowercase. Any characters that are not letters are unaffected by this function. Locale rules are applied if a locale is provided.
    LOWER,
    /// Inserts characters you specify to the left-side of a text string.
    LPAD,
    /// Returns the highest number from a list of numbers.
    MAX,
    /// Rounds a number up to the nearest integer, towards zero if negative.
    MCEILING,
    /// Rounds a number down to the nearest integer, away from zero if negative.
    MFLOOR,
    /// Returns the specified number of characters from the middle of a text string given the starting position.
    MID,
    /// Returns a milliseconds value in the form of a number from 0 through 999.
    MILLISECOND,
    /// Returns the lowest number from a list of numbers.
    MIN,
    /// Returns a minute value in the form of a number from 0 through 60.
    MINUTE,
    /// Returns a remainder after a number is divided by a specified divisor.
    MOD,
    /// Returns the month, a number from 1 (January) through 12 (December) in number format of a given date.
    MONTH,
    /// Returns FALSE for TRUE and TRUE for FALSE.
    NOT,
    /// Returns a date/time representing the current moment.
    NOW,
    /// Determines if an expression is null (blank) and returns a substitute expression if it is. If the expression is not blank, returns value of the expression.
    NULLVALUE,
    /// Determines if expressions are true or false. Returns TRUE if any expression is true. Returns FALSE if all expressions are false.
    OR,
    /// This function returns the value of a specified parent grouping. A “parent” grouping is any level above the one containing the formula.
    PARENTGROUPVAL,
    /// Returns an Einstein Discovery prediction for a record based on the specified record ID or for a list of fields and their values.
    PREDICT,
    /// This function returns the value of a specified previous grouping. A “previous” grouping is one that comes before the current grouping in the report.
    PREVGROUPVAL,
    /// Returns the previous value of a field.
    PRIORVALUE,
    /// Compares a text field to a regular expression and returns TRUE if there is a match. Otherwise, returns FALSE.
    REGEX,
    /// Returns a script tag with URL source that you specify. Use this function when referencing the Lightning Platform AJAX Toolkit or other JavaScript toolkits.
    REQUIRESCRIPT,
    /// Returns the characters of a source text string in reverse order.
    REVERSE,
    /// Returns the specified number of characters from the end of a text string.
    RIGHT,
    /// Returns the nearest number to a number you specify, constraining the new number by a specified number of digits.
    ROUND,
    /// Inserts characters that you specify to the right-side of a text string.
    RPAD,
    /// Returns a seconds value in the form of a number from 0 through 60.
    SECOND,
    /// Returns the positive square root of a given number.
    SQRT,
    /// Substitutes new text for old text in a text string.
    SUBSTITUTE,
    /// Converts a percent, number, date, date/time, or currency type field into text anywhere formulas are used. Also, converts picklist values to text in approval rules, approval step rules, workflow rules, escalation rules, assignment rules, auto-response rules, validation rules, formula fields, field updates, and custom buttons and links.
    TEXT,
    /// Returns a time value in GMT representing the current moment. Use this function instead of the NOW function if you only want to track time, without a date.
    TIMENOW,
    /// Returns the time value without the date, such as business hours.
    TIMEVALUE,
    /// Returns the current date as a date data type.
    TODAY,
    /// Removes the spaces and tabs from the beginning and end of a text string.
    TRIM,
    /// Converts all letters in the specified text string to uppercase.
    UPPER,
    /// Encodes text and merge field values for use in URLs by replacing characters that are illegal in URLs, such as blank spaces, with the code that represent those characters as defined in RFC 3986, Uniform Resource Identifier (URI): Generic Syntax.
    URLENCODE,
    /// Returns a URL for an action, an s-control, a Visualforce page, or a file in a static resource archive. URLFOR is available for use in custom buttons and links, s-controls, and Visualforce pages.
    URLFOR,
    /// Converts a text string to a number.
    VALUE,
    /// Returns a value by looking up a related value on a custom object similar to the VLOOKUP() Excel function. This function is only available in validation rules.
    VLOOKUP,
    /// Returns the day of the week for the given date, using 1 for Sunday, 2 for Monday, through 7 for Saturday.
    WEEKDAY,
    /// Returns the four-digit year in number format of a given date.
    YEAR,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArgumentList<'a>(pub Vec<Expression<'a>>);

impl<'a> FunctionArgumentList<'a> {
    pub fn add_argument(mut self, arg: Expression<'a>) -> Self {
        self.0.push(arg);
        self
    }
}

impl<'a, T: IntoIterator<Item = Expression<'a>>> From<T> for FunctionArgumentList<'a> {
    fn from(value: T) -> Self {
        Self(value.into_iter().collect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    /// Calculates the sum of two values.
    Add,
    /// Calculates the difference of two values.
    Subtract,
    /// Multiplies its values.
    Multiply,
    /// Divides its values.
    Divide,
    /// Raises a number to a power of a specified number.
    Exponentiation,
    /// Evaluates if two values are equivalent. The = and == operators are interchangeable.
    Equal,
    /// Evaluates if two values aren’t equivalent.
    NotEqual,
    /// Evaluates if a value is less than the value that follows this symbol.
    LessThan,
    /// Evaluates if a value is greater than the value that follows this symbol.
    GreaterThan,
    /// Evaluates if a value is less than or equal to the value that follows this symbol.
    LessThanOrEqual,
    /// Evaluates if a value is greater than or equal to the value that follows this symbol.
    GreaterThanOrEqual,
    /// Evaluates if two values or expressions are both true. Use this operator as an alternative to the logical function AND.
    And,
    /// Evaluates if at least one of multiple values or expressions is true. Use this operator as an alternative to the logical function OR.
    Or,
    /// Connects two or more strings.
    Concatenate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr<'a>(pub Expression<'a>, pub Operator, pub Expression<'a>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Comment<'a>(pub &'a str);

#[derive(Debug, Clone, PartialEq)]
pub struct Function<'a> {
    pub fname: FunctionName,
    pub args: FunctionArgumentList<'a>,
}

impl<'a> Function<'a> {
    pub fn new(fname: FunctionName, args: FunctionArgumentList<'a>) -> Self {
        Self { fname, args }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'a> {
    Function(Box<Function<'a>>),
    FieldRef(FieldReference<'a>),
    Literal(LiteralValue<'a>),
    BinaryExpr(Box<BinaryExpr<'a>>),
}

impl<'a> Expression<'a> {
    pub fn function(f: impl Into<Function<'a>>) -> Self {
        Self::Function(Box::new(f.into()))
    }

    pub fn field_ref(f: impl Into<FieldReference<'a>>) -> Self {
        Self::FieldRef(f.into())
    }

    pub fn literal(l: impl Into<LiteralValue<'a>>) -> Self {
        Self::Literal(l.into())
    }

    pub fn binary_expr(b: impl Into<BinaryExpr<'a>>) -> Self {
        Self::BinaryExpr(Box::new(b.into()))
    }
}

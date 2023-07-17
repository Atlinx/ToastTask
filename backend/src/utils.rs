use rocket::outcome::Outcome;

pub trait OkAsError<T, E> {
    fn ok_as_err(self) -> Result<E, T>;
}

impl<T, E> OkAsError<T, E> for Result<T, E> {
    fn ok_as_err(self) -> Result<E, T> {
        match self {
            Ok(ok) => Err(ok),
            Err(err) => Ok(err),
        }
    }
}

pub trait ResultAsOutcome<S, E, F> {
    fn as_outcome(self) -> Outcome<S, E, F>;
}

impl<S, E, F> ResultAsOutcome<S, E, F> for Result<S, E> {
    fn as_outcome(self) -> Outcome<S, E, F> {
        match self {
            Ok(s) => Outcome::Success(s),
            Err(e) => Outcome::Failure(e),
        }
    }
}


#[cfg(test)]
mod inputs {
    use nolik_cli::{Config, Flags, Flag, FlagKey, get_flag_values};
    use nolik_cli::inputs::errors::InputError;
    // use super::*;

    #[test]
    fn unrecognised_command() {
        let arr = ["unrecognised", "command"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::UnrecognisedCommand,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn not_enough_arguments() {
        let arr = ["argument".to_string()];
        let args = arr.iter();
        assert_eq!(
            InputError::NotEnoughArguments,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn no_arguments() {
        let arr = [];
        let args = arr.iter();
        assert_eq!(
            InputError::NoArguments,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn no_corresponding_value() {
        let arr = ["add", "asd", "--name"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::NoCorrespondingValue,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn required_key_missing() {
        let arr = ["add", "asd"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::RequiredKeysMissing,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn invalid_flags() {
        let arr = ["add", "asd", "--name", "alice", "--output", "value"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::InvalidFlag,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn non_unique_key() {
        let arr = ["add", "asd", "--name", "alice", "--name", "alice"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::NonUniqueKeys,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn non_unique_key_short_flags() {
        let arr = ["add", "asd", "-n", "alice", "-n", "alice"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::NonUniqueKeys,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn returns_flag_value() {
        let mut flags: Flags = Vec::new();
        let flag = Flag {
            key: FlagKey::Name,
            value: "alice".to_string(),
        };

        flags.push(flag);

        assert_eq!(
            vec![String::from("alice")],
            get_flag_values(FlagKey::Name, flags).unwrap(),
        );
    }
}

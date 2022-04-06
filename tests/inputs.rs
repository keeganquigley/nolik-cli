
#[cfg(test)]
mod inputs {
    use nolik_cli::inputs::errors::InputError;
    use nolik_cli::inputs::{Input, Flag, Flags, FlagKey, Command};

    #[test]
    fn unrecognised_command() {
        let arr = ["unrecognised", "command"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::UnrecognisedCommand,
            Input::new(args).unwrap_err()
        );
    }

    #[test]
    fn not_enough_arguments() {
        let arr = ["argument".to_string()];
        let args = arr.iter();
        assert_eq!(
            InputError::NotEnoughArguments,
            Input::new(args).unwrap_err()
        );
    }

    #[test]
    fn no_arguments() {
        let arr = [];
        let args = arr.iter();
        assert_eq!(
            InputError::NoArguments,
            Input::new(args).unwrap_err()
        );
    }

    #[test]
    fn no_corresponding_value() {
        let arr = ["add", "wallet", "--name"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::NoCorrespondingValue,
            Input::new(args).unwrap_err()
        );
    }

    #[test]
    fn required_key_missing() {
        let arr = ["add", "wallet"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::RequiredKeysMissing,
            Input::new(args).unwrap_err()
        );
    }

    #[test]
    fn invalid_flags() {
        let arr = ["add", "wallet", "--name", "alice", "--output", "value"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::InvalidFlag,
            Input::new(args).unwrap_err()
        );
    }

    #[test]
    fn non_unique_key() {
        let arr = ["add", "wallet", "--name", "alice", "--name", "alice"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::NonUniqueKeys,
            Input::new(args).unwrap_err()
        );
    }

    #[test]
    fn non_unique_key_short_flags() {
        let arr = ["add", "wallet", "-n", "alice", "-n", "alice"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            InputError::NonUniqueKeys,
            Input::new(args).unwrap_err()
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

        let input = Input {
            command: Command::AddWallet,
            flags,
        };

        assert_eq!(
            vec![String::from("alice")],
            input.get_flag_values(FlagKey::Name).unwrap(),
        );
    }
}

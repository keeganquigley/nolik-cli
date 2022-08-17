// use sodiumoxide::crypto::box_;
// use sodiumoxide::crypto::box_::{PublicKey, SecretKey};

// #[derive(Debug)]
// pub struct OneTimeUseNonce {
//     pub public: box_::Nonce,
//     pub secret: box_::Nonce,
// }
// 
// 
// impl OneTimeUseNonce {
//     pub(crate) fn new() -> OneTimeUseNonce {
//         OneTimeUseNonce {
//             public: box_::gen_nonce(),
//             secret: box_::gen_nonce(),
//         }
//     }
// }
// 
// 
// #[derive(Debug)]
// pub struct OneTimeUseBroker {
//     pub public: PublicKey,
//     pub secret: SecretKey,
// }
// 
// 
// impl OneTimeUseBroker {
//     fn new() -> OneTimeUseBroker {
//         let broker = box_::gen_keypair();
//         OneTimeUseBroker {
//             public: broker.0,
//             secret: broker.1,
//         }
//     }
// }
// 
// 
// #[derive(Debug)]
// pub struct OneTimeUse {
//     pub nonce: OneTimeUseNonce,
//     pub broker: OneTimeUseBroker,
// }
// 
// 
// impl OneTimeUse {
//     pub(crate) fn new() -> OneTimeUse {
//         OneTimeUse {
//             nonce: OneTimeUseNonce::new(),
//             broker: OneTimeUseBroker::new(),
//         }
//     }
// }

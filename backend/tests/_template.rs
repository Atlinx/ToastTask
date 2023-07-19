// #![cfg(test)]

// use crate::commons;
// use reqwest::StatusCode;

// #[rocket::async_test]
// async fn test() {
//     let client = commons::setup().await;

//     // INSERT CODE

// }

// macro_rules! test_macro {
//     ($($name:ident: $input:expr,)*) => {
//     $(
//         #[rocket::async_test]
//         async fn $name() {
//             let (json, status) = $input;
//             let client = commons::setup().await;
//             let res = client
//                 .post("/test/location")
//                 .json(&json)
//                 .send()
//                 .await
//                 .expect("Expected response");
//             assert_eq!(res.status(), status);
//         }
//     )*
//     }
// }

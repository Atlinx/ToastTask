// #![cfg(test)]

// use crate::commons;
// use reqwest::StatusCode;

// #[rocket::async_test]
// async fn test() {
//     let (client, app) = commons::setup().await;

//     // INSERT CODE

//     app.shutdown().await;
// }

// macro_rules! test_macro {
//     ($($name:ident: $input:expr,)*) => {
//     $(
//         #[rocket::async_test]
//         async fn $name() {
//             let (json, status) = $input;
//             let (client, app) = commons::setup().await;
//             let res = client
//                 .post("/test/location")
//                 .json(&json)
//                 .send()
//                 .await
//                 .expect("Expected response");
//             assert_eq!(res.status(), status);
//             app.shutdown().await;
//         }
//     )*
//     }
// }

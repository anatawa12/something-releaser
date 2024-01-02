macro_rules! put_server {
    [
        $server: ident (auth: $auth: literal) =>
        $($put_path: literal),* $(,)*
    ] => {
        {
            use httptest::matchers::*;
            use httptest::responders::*;
            use httptest::*;

            // prepare
            fn expectation(path: &'static str) -> Expectation {
                Expectation::matching(all_of![
                    request::headers(contains((
                        "authorization",
                        $auth
                    ))),
                    request::method_path("PUT", path),
                ])
                .respond_with(status_code(201))
            }

            $( $server.expect(expectation($put_path)); )*
        }
    };
}

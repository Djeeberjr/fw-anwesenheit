use dir_embed::Embed;
use picoserve::response::Content;

#[derive(Embed)]
#[dir = "../../web/dist"]
#[mode = "mime"]
pub struct Assets;

impl<State, CurrentPathParameters>
    picoserve::routing::PathRouterService<State, CurrentPathParameters> for Assets
{
    async fn call_request_handler_service<
        R: picoserve::io::embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        state: &State,
        current_path_parameters: CurrentPathParameters,
        path: picoserve::request::Path<'_>,
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        let requested_path = path.encoded();

        let requested_file = if requested_path == "/" {
            Self::get("index.html")
        } else if let Some(striped_path) = requested_path.strip_prefix("/") {
            Self::get(striped_path)
        } else {
            None
        };

        match requested_file {
            Some(content) => {
                let response = picoserve::response::Response::new(
                    picoserve::response::StatusCode::OK,
                    StaticAsset(content.0, content.1),
                );

                response_writer
                    .write_response(request.body_connection.finalize().await.unwrap(), response)
                    .await
            }
            None => {
                use picoserve::routing::PathRouter;
                picoserve::routing::NotFound
                    .call_path_router(
                        state,
                        current_path_parameters,
                        path,
                        request,
                        response_writer,
                    )
                    .await
            }
        }
    }
}

struct StaticAsset(pub &'static [u8], pub &'static str);

impl Content for StaticAsset {
    fn content_type(&self) -> &'static str {
        self.1
    }

    fn content_length(&self) -> usize {
        self.0.len()
    }

    async fn write_content<W: embedded_io_async::Write>(
        self,
        mut writer: W,
    ) -> Result<(), W::Error> {
        writer.write_all(self.0).await
    }
}

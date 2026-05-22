use crate::rust::GenRust;
use code_gen::rust::RustType;
use proto_packet_tree::{ModPathRef, Service, ServiceCall, WithTypeName};

impl GenRust<'_> {
    //! Gen Service: Router Text

    pub(super) fn gen_service_router_text(
        self,
        schema_path: ModPathRef<'_>,
        s: &Service,
    ) -> String {
        let trait_name: String = self.type_name(s);
        let route_prefix: String = format!("/{}.{}", schema_path.as_ref(), s.type_name());
        let mut out: String = String::new();
        out.push_str(&format!(
            "pub fn router<S>(service: ::std::sync::Arc<S>) -> ::axum::Router\nwhere\n    S: {} + 'static,\n{{\n    ::axum::Router::new()\n",
            trait_name,
        ));
        for call in s.calls() {
            out.push_str(&self.gen_service_router_route_text(&route_prefix, call));
        }
        out.push_str("}\n");
        out
    }

    fn gen_service_router_route_text(self, route_prefix: &str, call: &ServiceCall) -> String {
        let name: String = self.call_name(call);
        let request: RustType = self.field_type(call.request(), false, false);
        format!(
            "        .route(\n            \"{route_prefix}/{name}\",\n            ::axum::routing::post({{\n                let service = service.clone();\n                move |::axum::Json(request): ::axum::Json<{request}>| {{\n                    let service = service.clone();\n                    async move {{ service.{name}(request).await.map(::axum::Json) }}\n                }}\n            }}),\n        )\n"
        )
    }
}

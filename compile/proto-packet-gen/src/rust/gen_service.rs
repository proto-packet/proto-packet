use crate::rust::docs::add_docs;
use crate::rust::naming::{ident, is_upper_camel_case, snake_case};
use crate::rust::typing::Typing;
use code_gen::rust::{
    Access, Function, Receiver, RustType, Signature, SignatureDec, Trait, Var, WithAccess,
    WithAttributes, WithComments, WithFnGenerics, WithReceiver, WithResult, WithTraitFunctions,
    WithVarParams,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet_tree::{
    ModPathRef, Project, Service, ServiceCall, WithCallName, WithComments as _, WithTypeName,
};

/// Generates the Rust code of the `service` in the schema file at the `mod_path`.
///
/// The service is a trait with a function per call, plus a function that creates its axum router.
pub fn gen_service(project: &Project, mod_path: ModPathRef<'_>, service: &Service) -> String {
    let typing: Typing = Typing::new(project);
    Block::default()
        .with_statement(gen_trait(typing, service))
        .with_statement(EmptyLine::default())
        .with_statement(gen_router(typing, mod_path, service))
        .to_code()
}

/// Generates the trait of the `service`.
fn gen_trait(typing: Typing<'_>, service: &Service) -> Trait {
    let mut result: Trait = Trait::from(ident(&service.type_name()))
        .with_supertrait("::std::marker::Send")
        .with_supertrait("::std::marker::Sync");
    add_docs(&mut result, service.comments());
    if !is_upper_camel_case(&service.type_name()) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    for call in service.calls() {
        let mut signature_dec: SignatureDec = SignatureDec::from(gen_signature(typing, call));
        add_docs(&mut signature_dec, call.comments());
        result.add_signature_dec(signature_dec);
    }
    result
}

/// Generates the trait function signature of the `call`.
///
/// It returns a `Send` future, so the calls can be served by axum.
/// (ex: `fn greet(&self, request: Req) -> impl Future<Output = Result<Res, Error>> + Send + '_`)
fn gen_signature(typing: Typing<'_>, call: &ServiceCall) -> Signature {
    let result: RustType = RustType::from("::std::result::Result")
        .with_generic(typing.value_type(call.response()))
        .with_generic("::proto_packet::service::ServiceError");
    let future: RustType = RustType::from("::std::future::Future")
        .with_generic(result.to_binding("Output"))
        .to_impl()
        .with_bound("::std::marker::Send")
        .with_bound("'_");
    Signature::from(ident(&call.call_name()))
        .with_receiver(Receiver::Borrowed)
        .with_param(Var::from(("request", typing.value_type(call.request()))))
        .with_result(future)
}

/// Generates the function that creates the axum router of the `service`.
///
/// Each call is routed as a JSON `POST` to `/{mod_path}.{Service}/{call}`.
fn gen_router(typing: Typing<'_>, mod_path: ModPathRef<'_>, service: &Service) -> Function {
    let service_type: String = ident(&service.type_name());
    let param: &str = if service.calls().is_empty() {
        "_service"
    } else {
        "service"
    };
    let signature: Signature =
        Signature::from(format!("{}_router", snake_case(&service.type_name())))
            .with_generic(Var::from(("S", format!("{} + 'static", service_type))))
            .with_param(Var::from((
                param,
                RustType::from("::std::sync::Arc").with_generic("S"),
            )))
            .with_result("::proto_packet::axum::Router");
    let mut function: Function = Function::from(signature)
        .with_comment(format!(
            " Creates the axum router of the `{}` service.",
            service.type_name()
        ))
        .with_access(Access::Public)
        .with_literal("::proto_packet::axum::Router::new()");
    for call in service.calls() {
        let path: String = format!("/{}.{}/{}", mod_path, service.type_name(), call.call_name());
        function.add_literal(format!(
            ".route(\"{}\", ::proto_packet::axum::routing::post({{ \
                let service = service.clone(); \
                move |::proto_packet::axum::Json(request): ::proto_packet::axum::Json<{}>| {{ \
                    let service = service.clone(); \
                    async move {{ service.{}(request).await.map(::proto_packet::axum::Json) }} \
                }} \
            }}))",
            path,
            typing.value_type(call.request()),
            ident(&call.call_name())
        ));
    }
    function
}

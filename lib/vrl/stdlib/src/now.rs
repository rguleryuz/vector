use chrono::Utc;
use vrl::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Now;

impl Function for Now {
    fn identifier(&self) -> &'static str {
        "now"
    }

    fn examples(&self) -> &'static [Example] {
        &[Example {
            title: "now",
            source: r#"now() != """#,
            result: Ok("true"),
        }]
    }

    fn compile(
        &self,
        _state: (&mut state::LocalEnv, &mut state::ExternalEnv),
        _ctx: &mut FunctionCompileContext,
        _: ArgumentList,
    ) -> Compiled {
        Ok(Box::new(NowFn))
    }

    fn call_by_vm(&self, _ctx: &Context, _args: &mut VmArgumentList) -> Result<Value> {
        Ok(Utc::now().into())
    }
}

#[derive(Debug, Clone)]
struct NowFn;

impl Expression for NowFn {
    fn resolve<'value, 'ctx: 'value, 'rt: 'ctx>(
        &'rt self,
        _: &'ctx Context,
    ) -> Resolved<'value> {
        Ok(Cow::Owned(Utc::now().into()))
    }

    fn type_def(&self, _: (&state::LocalEnv, &state::ExternalEnv)) -> TypeDef {
        TypeDef::timestamp()
    }
}

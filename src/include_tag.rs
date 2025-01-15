use std::io::Write;

use liquid_core::error::ResultLiquidExt;
use liquid_core::model::KString;
use liquid_core::Expression;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::ValueView;
use liquid_core::{runtime::StackFrame, Runtime};
use liquid_core::{Error, Result};
use liquid_core::{ParseTag, TagReflection, TagTokenIter};

#[derive(Copy, Clone, Debug, Default)]
pub struct IncludeTag;

impl IncludeTag {
    pub fn new() -> Self {
        Self
    }
}

impl TagReflection for IncludeTag {
    fn tag(&self) -> &'static str {
        "include"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for IncludeTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        let partial = arguments.expect_next("Identifier or literal expected.")?;

        let partial = partial.expect_value().into_result()?;

        let mut vars: Vec<(KString, Expression)> = Vec::new();
        while let Ok(next) = arguments.expect_next("") {
            let id = next.expect_identifier().into_result()?.to_string();

            arguments
                .expect_next("\":\" expected.")?
                .expect_str(":")
                .into_result_custom_msg("expected \":\" to be used for the assignment")?;

            vars.push((
                id.into(),
                arguments
                    .expect_next("expected value")?
                    .expect_value()
                    .into_result()?,
            ));

            if let Ok(comma) = arguments.expect_next("") {
                // stop looking for variables if there is no comma
                // currently allows for one trailing comma
                if comma.expect_str(",").into_result().is_err() {
                    break;
                }
            }
        }

        arguments.expect_nothing()?;

        Ok(Box::new(Include { partial, vars }))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[derive(Debug)]
struct Include {
    partial: Expression,
    vars: Vec<(KString, Expression)>,
}

impl Renderable for Include {
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        let value = self.partial.evaluate(runtime)?;
        if !value.is_scalar() {
            return Error::with_msg("Can only `include` strings")
                .context("partial", format!("{}", value.source()))
                .into_err();
        }
        let name = value.to_kstr().into_owned();

        {
            // if there our additional variables creates a include object to access all the variables
            // from e.g. { include 'image.html' path="foo.png" }
            // then in image.html you could have <img src="{{include.path}}" />
            let mut pass_through = std::collections::HashMap::new();
            if !self.vars.is_empty() {
                for (id, val) in &self.vars {
                    let value = val
                        .try_evaluate(runtime)
                        .ok_or_else(|| Error::with_msg("failed to evaluate value"))?;

                    pass_through.insert(id.as_ref(), value);
                }
            }

            let scope = StackFrame::new(runtime, &pass_through);
            // let partial = scope
            //     .partials()
            //     .get(&name)
            //     .trace_with(|| format!("{{% include {} %}}", self.partial).into())?;

            println!("{}", name);
            let buf = std::fs::read_to_string(name).unwrap();
            writer.write(buf.as_bytes());

            // partial
            //     .render_to(writer, &scope)
            //     .trace_with(|| format!("{{% include {} %}}", self.partial).into())
            //     .context_key_with(|| self.partial.to_string().into())
            //     .value_with(|| name.to_string().into())?;
        }

        Ok(())
    }
}

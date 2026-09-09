#[macro_export]
macro_rules! export_command {
    ($cmd_name:ident, $cmd_type:ty, $data_type:ty, $err_type:ty) => {
        pub fn $cmd_name() -> $crate::commands::Command<$data_type, $err_type> {
            $crate::commands::Command {
                command: $crate::commands::TwilightCommand::from(<$cmd_type as ::twilight_interactions::command::CreateCommand>::create_command()).into(),
                handler: |ctx| {
                    ::futures_util::future::FutureExt::boxed(async move {
                        let command = <$cmd_type as ::twilight_interactions::command::CommandModel>::from_interaction(::twilight_model::application::interaction::application_command::CommandData::clone(&ctx.command_data).into())?;
                        command.handle(ctx).await?;
                        Ok(())
                    })
                },
            }
        }
    };
}

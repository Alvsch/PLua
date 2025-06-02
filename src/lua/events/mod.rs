use pumpkin::plugin::Context;

pub mod block_break;
pub mod block_place;
pub mod player_chat;
pub mod player_join;
pub mod player_leave;

pub async fn register_events(context: &Context) -> Result<(), String> {
    player_join::register(context).await?;
    player_leave::register(context).await?;
    player_chat::register(context).await?;
    block_place::register(context).await?;
    block_break::register(context).await?;

    Ok(())
}

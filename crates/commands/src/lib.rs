use twilight_interactions::command::CreateCommand;
use twilight_model::application::command::Command;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProxyProtocol {
    Http,
    Https,
    Socks4,
    Socks5,
    All,
}

impl ProxyProtocol {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Http => "HTTP",
            Self::Https => "HTTPS",
            Self::Socks4 => "SOCKS4",
            Self::Socks5 => "SOCKS5",
            Self::All => "URL",
        }
    }

    #[must_use]
    pub const fn filename(self) -> &'static str {
        match self {
            Self::Http => "http.txt",
            Self::Https => "https.txt",
            Self::Socks4 => "socks4.txt",
            Self::Socks5 => "socks5.txt",
            Self::All => "proxies.txt",
        }
    }

    #[must_use]
    pub const fn attachment_description(self) -> &'static str {
        match self {
            Self::Http => "HTTP Proxies",
            Self::Https => "HTTPS Proxies",
            Self::Socks4 => "SOCKS4 Proxies",
            Self::Socks5 => "SOCKS5 Proxies",
            Self::All => "URL Proxies",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BotCommand {
    Proxy(ProxyProtocol),
    Invite,
}

impl BotCommand {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            HttpCommand::NAME => Some(Self::Proxy(ProxyProtocol::Http)),
            HttpsCommand::NAME => Some(Self::Proxy(ProxyProtocol::Https)),
            Socks4Command::NAME => Some(Self::Proxy(ProxyProtocol::Socks4)),
            Socks5Command::NAME => Some(Self::Proxy(ProxyProtocol::Socks5)),
            AllCommand::NAME => Some(Self::Proxy(ProxyProtocol::All)),
            InviteCommand::NAME => Some(Self::Invite),
            _ => None,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Proxy(ProxyProtocol::Http) => HttpCommand::NAME,
            Self::Proxy(ProxyProtocol::Https) => HttpsCommand::NAME,
            Self::Proxy(ProxyProtocol::Socks4) => Socks4Command::NAME,
            Self::Proxy(ProxyProtocol::Socks5) => Socks5Command::NAME,
            Self::Proxy(ProxyProtocol::All) => AllCommand::NAME,
            Self::Invite => InviteCommand::NAME,
        }
    }
}

#[derive(CreateCommand)]
#[command(
    name = "http",
    desc = "Post http proxies to this channel.",
    contexts = "guild bot_dm private_channel"
)]
struct HttpCommand;

#[derive(CreateCommand)]
#[command(
    name = "https",
    desc = "Post https proxies to this channel.",
    contexts = "guild bot_dm private_channel"
)]
struct HttpsCommand;

#[derive(CreateCommand)]
#[command(
    name = "socks4",
    desc = "Post socks4 proxies to this channel.",
    contexts = "guild bot_dm private_channel"
)]
struct Socks4Command;

#[derive(CreateCommand)]
#[command(
    name = "socks5",
    desc = "Post socks5 proxies to this channel.",
    contexts = "guild bot_dm private_channel"
)]
struct Socks5Command;

#[derive(CreateCommand)]
#[command(
    name = "all",
    desc = "Post http, https, socks4 and socks5 proxies to this channel.",
    contexts = "guild bot_dm private_channel"
)]
struct AllCommand;

#[derive(CreateCommand)]
#[command(
    name = "invite",
    desc = "Get a link to install this app in your server or DMs.",
    contexts = "guild bot_dm private_channel"
)]
struct InviteCommand;

#[must_use]
pub fn global_commands() -> Vec<Command> {
    [
        HttpCommand::create_command(),
        HttpsCommand::create_command(),
        Socks4Command::create_command(),
        Socks5Command::create_command(),
        AllCommand::create_command(),
        InviteCommand::create_command(),
    ]
    .into_iter()
    .map(Into::into)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{BotCommand, ProxyProtocol, global_commands};
    use twilight_model::application::{command::CommandType, interaction::InteractionContextType};

    const COMMANDS: [BotCommand; 6] = [
        BotCommand::Proxy(ProxyProtocol::Http),
        BotCommand::Proxy(ProxyProtocol::Https),
        BotCommand::Proxy(ProxyProtocol::Socks4),
        BotCommand::Proxy(ProxyProtocol::Socks5),
        BotCommand::Proxy(ProxyProtocol::All),
        BotCommand::Invite,
    ];

    #[test]
    fn command_names_round_trip() {
        for command in COMMANDS {
            assert_eq!(BotCommand::from_name(command.name()), Some(command));
        }
        assert_eq!(BotCommand::from_name("unknown"), None);
    }

    #[test]
    fn generated_commands_support_every_install_context() {
        let expected_contexts = vec![
            InteractionContextType::Guild,
            InteractionContextType::BotDm,
            InteractionContextType::PrivateChannel,
        ];
        let commands = global_commands();

        assert_eq!(commands.len(), COMMANDS.len());
        for (definition, command) in commands.iter().zip(COMMANDS) {
            assert_eq!(definition.name, command.name());
            assert_eq!(definition.kind, CommandType::ChatInput);
            assert_eq!(definition.contexts.as_ref(), Some(&expected_contexts));
            assert!(definition.options.is_empty());
        }
    }

    #[test]
    fn proxy_protocols_have_matching_attachment_metadata() {
        let cases = [
            (ProxyProtocol::Http, "HTTP", "http.txt", "HTTP Proxies"),
            (ProxyProtocol::Https, "HTTPS", "https.txt", "HTTPS Proxies"),
            (
                ProxyProtocol::Socks4,
                "SOCKS4",
                "socks4.txt",
                "SOCKS4 Proxies",
            ),
            (
                ProxyProtocol::Socks5,
                "SOCKS5",
                "socks5.txt",
                "SOCKS5 Proxies",
            ),
            (ProxyProtocol::All, "URL", "proxies.txt", "URL Proxies"),
        ];

        for (protocol, label, filename, description) in cases {
            assert_eq!(protocol.label(), label);
            assert_eq!(protocol.filename(), filename);
            assert_eq!(protocol.attachment_description(), description);
        }
    }
}

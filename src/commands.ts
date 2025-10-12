import {
  ApplicationCommandType,
  InteractionContextType,
  type RESTPostAPIApplicationCommandsJSONBody,
} from "discord-api-types/v10";

export const HTTP_COMMAND: RESTPostAPIApplicationCommandsJSONBody = {
  name: "http",
  description: "Post http proxies to this channel.",
  type: ApplicationCommandType.ChatInput,
  contexts: [
    InteractionContextType.Guild,
    InteractionContextType.BotDM,
    InteractionContextType.PrivateChannel,
  ],
};

export const HTTPS_COMMAND: RESTPostAPIApplicationCommandsJSONBody = {
  name: "https",
  description: "Post https proxies to this channel.",
  type: ApplicationCommandType.ChatInput,
  contexts: [
    InteractionContextType.Guild,
    InteractionContextType.BotDM,
    InteractionContextType.PrivateChannel,
  ],
};

export const SOCKS4_COMMAND: RESTPostAPIApplicationCommandsJSONBody = {
  name: "socks4",
  description: "Post socks4 proxies to this channel.",
  type: ApplicationCommandType.ChatInput,
  contexts: [
    InteractionContextType.Guild,
    InteractionContextType.BotDM,
    InteractionContextType.PrivateChannel,
  ],
};

export const SOCKS5_COMMAND: RESTPostAPIApplicationCommandsJSONBody = {
  name: "socks5",
  description: "Post socks5 proxies to this channel.",
  type: ApplicationCommandType.ChatInput,
  contexts: [
    InteractionContextType.Guild,
    InteractionContextType.BotDM,
    InteractionContextType.PrivateChannel,
  ],
};

export const ALL_COMMAND: RESTPostAPIApplicationCommandsJSONBody = {
  name: "all",
  description: "Post http, https, socks4 and socks5 proxies to this channel.",
  type: ApplicationCommandType.ChatInput,
  contexts: [
    InteractionContextType.Guild,
    InteractionContextType.BotDM,
    InteractionContextType.PrivateChannel,
  ],
};

export const INVITE_COMMAND: RESTPostAPIApplicationCommandsJSONBody = {
  name: "invite",
  description: "Get a link to install this app in your server or DMs.",
  type: ApplicationCommandType.ChatInput,
  contexts: [
    InteractionContextType.Guild,
    InteractionContextType.BotDM,
    InteractionContextType.PrivateChannel,
  ],
};

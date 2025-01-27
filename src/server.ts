/**
 * The core server that runs on a Cloudflare worker.
 */

import {AutoRouter, IRequest} from 'itty-router';
import {ALL_COMMAND, HTTP_COMMAND, HTTPS_COMMAND, INVITE_COMMAND, SOCKS4_COMMAND, SOCKS5_COMMAND} from './commands';
import {
  APIInteraction,
  APIInteractionResponse,
  ChannelType,
  InteractionResponseType,
  InteractionType,
  MessageFlags
} from "discord-api-types/v10";
import {verifyKey} from "discord-interactions";

class FormDataResponse extends Response {
  constructor(body: APIInteractionResponse | {
    error: string
  }, extras: {
    blob: Blob,
    name: string,
    fileName: string,
  }[]) {
    const formData = new FormData();
    formData.set('payload_json', new Blob([JSON.stringify(body)], {
      type: 'application/json',
    }));
    for (const extra of extras) {
      formData.set(extra.name, extra.blob, extra.fileName);
    }

    super(formData);
  }
}

class JsonResponse extends Response {
  constructor(body: APIInteractionResponse | {
    error: string
  }, init?: ResponseInit) {
    super(JSON.stringify(body), init || {
      headers: {
        'content-type': 'application/json;charset=UTF-8',
      },
    });
  }
}

const router = AutoRouter();

/**
 * A simple :wave: hello page to verify the worker is working.
 */
router.get('/', (request, env: Env) => {
  return new Response(`👋 ${env.DISCORD_APPLICATION_ID}`);
});

type ProxyAPIResponse = {
  success: false
} | {
  success: true
  update_time: number,
  count: number,
  http: string[],
  https: string[],
  socks4: string[],
  socks5: string[],
}

async function fetchProxies(env: Env): Promise<ProxyAPIResponse> {
  const response = await fetch(env.PROXY_API_URL, {
    cf: {
      // 2 hours
      cacheTtl: 2 * 60 * 60,
      cacheEverything: true,
    }
  });

  if (!response.ok) {
    return {success: false};
  }

  return await response.json();
}

/**
 * Main route for all requests sent from Discord.  All incoming messages will
 * include a JSON payload described here:
 * https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object
 */
router.post('/interactions', async (request, env: Env) => {
  const parsedRequest = await verifyDiscordRequest(
      request,
      env,
  );
  if (!parsedRequest.isValid) {
    return new Response('Bad request signature.', {status: 401});
  }

  const {interaction} = parsedRequest;

  if (interaction.type === InteractionType.Ping) {
    // The `PING` message is used during the initial webhook handshake, and is
    // required to configure the webhook in the developer portal.
    return new JsonResponse({
      type: InteractionResponseType.Pong,
    });
  }

  if (interaction.type === InteractionType.ApplicationCommand) {
    if (interaction.data.name === INVITE_COMMAND.name) {
      const applicationId = env.DISCORD_APPLICATION_ID;
      const INVITE_URL = `https://discord.com/oauth2/authorize?client_id=${applicationId}`;
      return new JsonResponse({
        type: InteractionResponseType.ChannelMessageWithSource,
        data: {
          content: INVITE_URL,
          flags: MessageFlags.Ephemeral,
        },
      });
    }

    if (interaction.channel.type !== ChannelType.DM
        && interaction.channel.type !== ChannelType.GroupDM
        && interaction.channel.name !== "proxy") {
      return new JsonResponse({
        type: InteractionResponseType.ChannelMessageWithSource,
        data: {
          content: 'This command can only be used in a #proxy channel.',
          flags: MessageFlags.Ephemeral,
        },
      });
    }

    const proxies = await fetchProxies(env);
    if (!proxies.success) {
      return new JsonResponse({
        type: InteractionResponseType.ChannelMessageWithSource,
        data: {
          content: 'Failed to fetch proxies.',
          flags: MessageFlags.Ephemeral,
        },
      });
    }

    switch (interaction.data.name.toLowerCase()) {
      case HTTP_COMMAND.name.toLowerCase(): {
        const proxyList = proxies.http.join('\n');
        return new FormDataResponse({
          type: InteractionResponseType.ChannelMessageWithSource,
          data: {
            content: "Here are your HTTP proxies!",
            attachments: [{
              id: 0,
              filename: 'http.txt',
              description: 'HTTP Proxies',
            }]
          },
        }, [{
          blob: new Blob([proxyList], {type: 'text/plain'}),
          name: 'files[0]',
          fileName: 'http.txt',
        }]);
      }
      case HTTPS_COMMAND.name.toLowerCase(): {
        const proxyList = proxies.https.join('\n');
        return new FormDataResponse({
          type: InteractionResponseType.ChannelMessageWithSource,
          data: {
            content: "Here are your HTTPS proxies!",
            attachments: [{
              id: 0,
              filename: 'https.txt',
              description: 'HTTPS Proxies',
            }]
          },
        }, [{
          blob: new Blob([proxyList], {type: 'text/plain'}),
          name: 'files[0]',
          fileName: 'https.txt',
        }]);
      }
      case SOCKS4_COMMAND.name.toLowerCase(): {
        const proxyList = proxies.socks4.join('\n');
        return new FormDataResponse({
          type: InteractionResponseType.ChannelMessageWithSource,
          data: {
            content: "Here are your SOCKS4 proxies!",
            attachments: [{
              id: 0,
              filename: 'socks4.txt',
              description: 'SOCKS4 Proxies',
            }]
          },
        }, [{
          blob: new Blob([proxyList], {type: 'text/plain'}),
          name: 'files[0]',
          fileName: 'socks4.txt',
        }]);
      }
      case SOCKS5_COMMAND.name.toLowerCase(): {
        const proxyList = proxies.socks5.join('\n');
        return new FormDataResponse({
          type: InteractionResponseType.ChannelMessageWithSource,
          data: {
            content: "Here are your SOCKS5 proxies!",
            attachments: [{
              id: 0,
              filename: 'socks5.txt',
              description: 'SOCKS5 Proxies',
            }]
          },
        }, [{
          blob: new Blob([proxyList], {type: 'text/plain'}),
          name: 'files[0]',
          fileName: 'socks5.txt',
        }]);
      }
      case ALL_COMMAND.name.toLowerCase(): {
        // noinspection HttpUrlsUsage
        const proxyList = [
          proxies.http.map(url => `http://${url}`).join('\n'),
          proxies.https.map(url => `https://${url}`).join('\n'),
          proxies.socks4.map(url => `socks4://${url}`).join('\n'),
          proxies.socks5.map(url => `socks5://${url}`).join('\n'),
        ].join("\n");
        return new FormDataResponse({
          type: InteractionResponseType.ChannelMessageWithSource,
          data: {
            content: "Here are your URL proxies!",
            attachments: [{
              id: 0,
              filename: 'proxies.txt',
              description: 'URL Proxies',
            }]
          },
        }, [{
          blob: new Blob([proxyList], {type: 'text/plain'}),
          name: 'files[0]',
          fileName: 'proxies.txt',
        }]);
      }
      default:
        return new JsonResponse({error: 'Unknown Type'}, {status: 400});
    }
  }

  return new JsonResponse({error: 'Unknown Type'}, {status: 400});
});
router.all('*', () => new Response('Not Found.', {status: 404}));

export interface Env {
  DISCORD_APPLICATION_ID: string
  DISCORD_PUBLIC_KEY: string
  DISCORD_TOKEN: string
  PROXY_API_URL: string
}

async function verifyDiscordRequest(request: IRequest, env: Env): Promise<{
  isValid: boolean
  interaction: APIInteraction
}> {
  const signature = request.headers.get('x-signature-ed25519');
  const timestamp = request.headers.get('x-signature-timestamp');
  const body = await request.bytes();
  const isValidRequest =
      signature &&
      timestamp &&
      (await verifyKey(body, signature, timestamp, env.DISCORD_PUBLIC_KEY));
  const parsedBody = JSON.parse(new TextDecoder().decode(body));

  return {interaction: parsedBody, isValid: !!isValidRequest};
}

export default {
  fetch: router.fetch,
} satisfies ExportedHandler<Env>;

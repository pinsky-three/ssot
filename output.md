
# vacuul-dev


## vacuul-frontend
`clone_url`: https://github.com/vacuul-dev/vacuul-frontend.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-frontend/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 301   


``````
# vacuul-frontend
This repository contains the frontend code for the Vacuul app, a platform designed to let the user to set the configurations for the hardware. Built using the Tailwind environment, it provides a responsive and user-friendly interface for users to interact with the app’s features.

``````



## vacuul-platform
`clone_url`: https://github.com/vacuul-dev/vacuul-platform.git


### fresh.config.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/fresh.config.ts
`relative_path`: fresh.config.ts
`format`: Arbitrary Binary Data
`size`: 769   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import tailwind from "$fresh/plugins/tailwind.ts";
import kvOAuthPlugin from "./plugins/kv_oauth.ts";
import sessionPlugin from "./plugins/session.ts";
import errorHandling from "./plugins/error_handling.ts";
import securityHeaders from "./plugins/security_headers.ts";
import welcomePlugin from "./plugins/welcome.ts";
import type { FreshConfig } from "$fresh/server.ts";
import { ga4Plugin } from "https://deno.land/x/fresh_ga4@0.0.4/mod.ts";
import { blog } from "./plugins/blog/mod.ts";

export default {
  plugins: [
    ga4Plugin(),
    welcomePlugin,
    kvOAuthPlugin,
    sessionPlugin,
    tailwind(),
    errorHandling,
    securityHeaders,
    blog(),
  ],
} satisfies FreshConfig;

``````


### tasks/init_stripe.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/tasks/init_stripe.ts
`relative_path`: tasks/init_stripe.ts
`format`: Arbitrary Binary Data
`size`: 1971   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type Stripe from "stripe";
import { SITE_DESCRIPTION } from "@/utils/constants.ts";
import { isStripeEnabled, stripe } from "@/utils/stripe.ts";

async function createPremiumTierProduct(stripe: Stripe) {
  /**
   * These values provide a set of default values for the demo.
   * However, these can be adjusted to fit your use case.
   */
  return await stripe.products.create({
    name: "Premium",
    description: "Unlock premium features like flair and more.",
    default_price_data: {
      unit_amount: 500,
      currency: "usd",
      recurring: {
        interval: "month",
      },
    },
  });
}

async function createDefaultPortalConfiguration(
  stripe: Stripe,
  product:
    Stripe.BillingPortal.ConfigurationCreateParams.Features.SubscriptionUpdate.Product,
) {
  return await stripe.billingPortal.configurations.create({
    features: {
      payment_method_update: {
        enabled: true,
      },
      customer_update: {
        allowed_updates: ["email", "name"],
        enabled: true,
      },
      subscription_cancel: {
        enabled: true,
        mode: "immediately",
      },
      subscription_update: {
        enabled: true,
        default_allowed_updates: ["price"],
        products: [product],
      },
      invoice_history: { enabled: true },
    },
    business_profile: {
      headline: SITE_DESCRIPTION,
    },
  });
}

async function main() {
  if (!isStripeEnabled()) throw new Error("Stripe is disabled.");

  const product = await createPremiumTierProduct(stripe);

  if (typeof product.default_price !== "string") return;

  await createDefaultPortalConfiguration(stripe, {
    prices: [product.default_price],
    product: product.id,
  });

  console.log(
    "Please copy and paste this value into the `STRIPE_PREMIUM_PLAN_PRICE_ID` variable in `.env`: " +
      product.default_price,
  );
}

if (import.meta.main) {
  await main();
}

``````


### tasks/db_seed.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/tasks/db_seed.ts
`relative_path`: tasks/db_seed.ts
`format`: Arbitrary Binary Data
`size`: 1694   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
// Description: Seeds the kv db with Hacker News stories
import { createItem, createUser } from "@/utils/db.ts";
import { ulid } from "$std/ulid/mod.ts";

// Reference: https://github.com/HackerNews/API
const API_BASE_URL = `https://hacker-news.firebaseio.com/v0`;
const API_ITEM_URL = `${API_BASE_URL}/item`;
const API_TOP_STORIES_URL = `${API_BASE_URL}/topstories.json`;
const TOP_STORIES_COUNT = 10;

interface Story {
  id: number;
  score: number;
  time: number; // Unix seconds
  by: string;
  title: string;
  url: string;
}

const resp = await fetch(API_TOP_STORIES_URL);
const allTopStories = await resp.json() as number[];
const topStories = allTopStories.slice(0, TOP_STORIES_COUNT);
const storiesPromises = [];

for (const id of topStories) {
  storiesPromises.push(fetch(`${API_ITEM_URL}/${id}.json`));
}

const storiesResponses = await Promise.all(storiesPromises);
const stories = await Promise.all(
  storiesResponses.map((r) => r.json()),
) as Story[];
const items = stories.map(({ by: userLogin, title, url, score, time }) => ({
  id: ulid(),
  userLogin,
  title,
  url,
  score,
  createdAt: new Date(time * 1000),
})).filter(({ url }) => url);

const users = new Set(items.map((user) => user.userLogin));

const itemPromises = [];
for (const item of items) {
  itemPromises.push(createItem(item));
}
await Promise.all(itemPromises);

const userPromises = [];
for (const login of users) {
  userPromises.push(
    createUser({
      login,
      stripeCustomerId: crypto.randomUUID(),
      sessionId: crypto.randomUUID(),
      isSubscribed: false,
    }),
  );
}
await Promise.all(userPromises);

``````


### tasks/db_dump.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/tasks/db_dump.ts
`relative_path`: tasks/db_dump.ts
`format`: Arbitrary Binary Data
`size`: 668   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
/**
 * This script prints all entries in the KV database formatted as JSON. This
 * can be used to create a backup file.
 *
 * @example
 * ```bash
 * deno task db:dump > backup.json
 * ```
 */
import { kv } from "@/utils/db.ts";

// https://github.com/GoogleChromeLabs/jsbi/issues/30#issuecomment-521460510
function replacer(_key: unknown, value: unknown) {
  return typeof value === "bigint" ? value.toString() : value;
}

const items = await Array.fromAsync(
  kv.list({ prefix: [] }),
  ({ key, value }) => ({ key, value }),
);
console.log(JSON.stringify(items, replacer, 2));

kv.close();

``````


### tasks/check_license.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/tasks/check_license.ts
`relative_path`: tasks/check_license.ts
`format`: Arbitrary Binary Data
`size`: 2015   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
// Copied from std/_tools/check_license.ts

import { walk } from "$std/fs/walk.ts";
import { globToRegExp } from "$std/path/glob_to_regexp.ts";

const EXTENSIONS = [".ts", ".tsx"];
const EXCLUDED_DIRS = [
  "data",
  "static",
];

const ROOT = new URL("../", import.meta.url);
const CHECK = Deno.args.includes("--check");
const CURRENT_YEAR = new Date().getFullYear();
const RX_COPYRIGHT = new RegExp(
  `// Copyright 2023-([0-9]{4}) the Deno authors\\. All rights reserved\\. MIT license\\.\\n`,
  "m",
);
const COPYRIGHT =
  `// Copyright 2023-${CURRENT_YEAR} the Deno authors. All rights reserved. MIT license.`;

let failed = false;

for await (
  const { path } of walk(ROOT, {
    exts: EXTENSIONS,
    skip: [
      ...EXCLUDED_DIRS.map((path) => globToRegExp(ROOT.pathname + path)),
      new RegExp("fresh.gen.ts"),
    ],
    includeDirs: false,
  })
) {
  const content = await Deno.readTextFile(path);
  const match = content.match(RX_COPYRIGHT);

  if (!match) {
    if (CHECK) {
      console.error(`Missing copyright header: ${path}`);
      failed = true;
    } else {
      const contentWithCopyright = COPYRIGHT + "\n" + content;
      await Deno.writeTextFile(path, contentWithCopyright);
      console.log("Copyright header automatically added to " + path);
    }
  } else if (parseInt(match[1]) !== CURRENT_YEAR) {
    if (CHECK) {
      console.error(`Incorrect copyright year: ${path}`);
      failed = true;
    } else {
      const index = match.index ?? 0;
      const contentWithoutCopyright = content.replace(match[0], "");
      const contentWithCopyright = contentWithoutCopyright.substring(0, index) +
        COPYRIGHT + "\n" + contentWithoutCopyright.substring(index);
      await Deno.writeTextFile(path, contentWithCopyright);
      console.log("Copyright header automatically updated in " + path);
    }
  }
}

if (failed) {
  console.info(`Copyright header should be "${COPYRIGHT}"`);
  Deno.exit(1);
}

``````


### tasks/db_reset.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/tasks/db_reset.ts
`relative_path`: tasks/db_reset.ts
`format`: Arbitrary Binary Data
`size`: 356   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { kv } from "@/utils/db.ts";

if (!confirm("WARNING: The database will be reset. Continue?")) Deno.exit();

const iter = kv.list({ prefix: [] });
const promises = [];
for await (const res of iter) promises.push(kv.delete(res.key));
await Promise.all(promises);

kv.close();

``````


### tasks/db_restore.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/tasks/db_restore.ts
`relative_path`: tasks/db_restore.ts
`format`: Arbitrary Binary Data
`size`: 1108   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
/**
 * This script is used to restore a KV database by a file generated by the dump
 * script.
 *
 * @example
 * ```bash
 * deno task db:restore backup.json
 * ```
 */
import { kv } from "@/utils/db.ts";

interface StoredKvU64 {
  value: string;
}

function isStoredKvU64(value: unknown): value is StoredKvU64 {
  return (value as StoredKvU64).value !== undefined &&
    typeof (value as StoredKvU64).value === "string";
}

function reviver(_key: unknown, value: unknown) {
  return isStoredKvU64(value) ? new Deno.KvU64(BigInt(value.value)) : value;
}

if (!confirm("WARNING: The database will be restored. Continue?")) Deno.exit();

const [filePath] = Deno.args;
if (filePath === undefined) throw new Error("File path must be defined");

const rawEntries = Deno.readTextFileSync(filePath);
const entries = JSON.parse(rawEntries, reviver) as Omit<
  Deno.KvEntry<unknown>,
  "versionstamp"
>[];

const promises = [];
for (const { key, value } of entries) promises.push(kv.set(key, value));
await Promise.all(promises);

kv.close();

``````


### tasks/db_migrate.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/tasks/db_migrate.ts
`relative_path`: tasks/db_migrate.ts
`format`: Arbitrary Binary Data
`size`: 443   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
/**
 * This script is used to perform migration jobs on the database. These jobs
 * can be performed on remote KV instances using
 * {@link https://github.com/denoland/deno/tree/main/ext/kv#kv-connect|KV Connect}.
 *
 * This script will continually change over time for database migrations, as
 * required.
 *
 * @example
 * ```bash
 * deno task db:migrate
 * ```
 */

``````


### types/therapy.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/types/therapy.ts
`relative_path`: types/therapy.ts
`format`: Arbitrary Binary Data
`size`: 287   


``````
export interface TherapyOption {
  id: string;
  name: string;
  lightColor: string;
  temperatureRange: {
    min: number;
    max: number;
  };
  description: string;
  duration: number;
}

export interface CustomTherapy extends Omit<TherapyOption, "id" | "name"> {
  name?: string;
}

``````


### posts/first-post.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/posts/first-post.md
`relative_path`: posts/first-post.md
`format`: Arbitrary Binary Data
`size`: 207   


``````
---
title: This is my first blog post!
publishedAt: 2022-11-04T15:00:00.000Z
summary: This is an excerpt of my first blog post.
---

# Heading 1

Hello, world!

```javascript
console.log("Hello World");
```

``````


### posts/second-post.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/posts/second-post.md
`relative_path`: posts/second-post.md
`format`: Arbitrary Binary Data
`size`: 1378   


``````
---
title: Second post
publishedAt: 2022-11-04T15:00:00.000Z
summary: Lorem Ipsum is simply dummy text of the printing and typesetting industry.
---

It was popularised in the 1960s with the release of Letraset sheets containing
Lorem Ipsum passages, and more recently with desktop publishing software like
Aldus PageMaker including versions of Lorem Ipsum.

Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem
Ipsum has been the industry's standard dummy text ever since the 1500s, when an
unknown printer took a galley of type and scrambled it to make a type specimen
book. It has survived not only five centuries, but also the leap into electronic
typesetting, remaining essentially unchanged. It was popularised in the 1960s
with the release of Letraset sheets containing Lorem Ipsum passages, and more
recently with desktop publishing software like Aldus PageMaker including
versions of Lorem Ipsum.

## Usage

```js
import blog from "https://deno.land/x/blog/blog.tsx";

blog({
  author: "Dino",
  title: "My Blog",
  description: "The blog description.",
  avatar: "https://deno-avatar.deno.dev/avatar/blog.svg",
  avatarClass: "rounded-full",
  links: [
    { title: "Email", url: "mailto:bot@deno.com" },
    { title: "GitHub", url: "https://github.com/denobot" },
    { title: "Twitter", url: "https://twitter.com/denobot" },
  ],
});
```

``````


### fresh.gen.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/fresh.gen.ts
`relative_path`: fresh.gen.ts
`format`: Arbitrary Binary Data
`size`: 4161   


``````
// DO NOT EDIT. This file is generated by Fresh.
// This file SHOULD be checked into source version control.
// This file is automatically updated during development when running `dev.ts`.

import * as $_404 from "./routes/_404.tsx";
import * as $_500 from "./routes/_500.tsx";
import * as $_app from "./routes/_app.tsx";
import * as $account_index from "./routes/account/index.tsx";
import * as $account_manage from "./routes/account/manage.ts";
import * as $account_upgrade from "./routes/account/upgrade.ts";
import * as $api_items_id_ from "./routes/api/items/[id].ts";
import * as $api_items_index from "./routes/api/items/index.ts";
import * as $api_me_votes from "./routes/api/me/votes.ts";
import * as $api_stripe_webhooks from "./routes/api/stripe-webhooks.ts";
import * as $api_users_login_index from "./routes/api/users/[login]/index.ts";
import * as $api_users_login_items from "./routes/api/users/[login]/items.ts";
import * as $api_users_index from "./routes/api/users/index.ts";
import * as $api_vote from "./routes/api/vote.ts";
import * as $booking_payment from "./routes/booking/payment.tsx";
import * as $booking_success from "./routes/booking/success.tsx";
import * as $dashboard_index from "./routes/dashboard/index.tsx";
import * as $dashboard_stats from "./routes/dashboard/stats.tsx";
import * as $dashboard_users from "./routes/dashboard/users.tsx";
import * as $end_of_therapy from "./routes/end-of-therapy.tsx";
import * as $index from "./routes/index.tsx";
import * as $login from "./routes/login.tsx";
import * as $machine_control from "./routes/machine-control.tsx";
import * as $pricing from "./routes/pricing.tsx";
import * as $submit from "./routes/submit.tsx";
import * as $therapy_progress from "./routes/therapy-progress.tsx";
import * as $therapy_therapyId_ from "./routes/therapy/[therapyId].tsx";
import * as $therapy_index from "./routes/therapy/index.tsx";
import * as $user_profile from "./routes/user-profile.tsx";
import * as $users_login_ from "./routes/users/[login].tsx";
import * as $welcome from "./routes/welcome.tsx";
import * as $Chart from "./islands/Chart.tsx";
import * as $ItemsList from "./islands/ItemsList.tsx";
import * as $TherapyProgress from "./islands/TherapyProgress.tsx";
import * as $UsersTable from "./islands/UsersTable.tsx";
import { type Manifest } from "$fresh/server.ts";

const manifest = {
  routes: {
    "./routes/_404.tsx": $_404,
    "./routes/_500.tsx": $_500,
    "./routes/_app.tsx": $_app,
    "./routes/account/index.tsx": $account_index,
    "./routes/account/manage.ts": $account_manage,
    "./routes/account/upgrade.ts": $account_upgrade,
    "./routes/api/items/[id].ts": $api_items_id_,
    "./routes/api/items/index.ts": $api_items_index,
    "./routes/api/me/votes.ts": $api_me_votes,
    "./routes/api/stripe-webhooks.ts": $api_stripe_webhooks,
    "./routes/api/users/[login]/index.ts": $api_users_login_index,
    "./routes/api/users/[login]/items.ts": $api_users_login_items,
    "./routes/api/users/index.ts": $api_users_index,
    "./routes/api/vote.ts": $api_vote,
    "./routes/booking/payment.tsx": $booking_payment,
    "./routes/booking/success.tsx": $booking_success,
    "./routes/dashboard/index.tsx": $dashboard_index,
    "./routes/dashboard/stats.tsx": $dashboard_stats,
    "./routes/dashboard/users.tsx": $dashboard_users,
    "./routes/end-of-therapy.tsx": $end_of_therapy,
    "./routes/index.tsx": $index,
    "./routes/login.tsx": $login,
    "./routes/machine-control.tsx": $machine_control,
    "./routes/pricing.tsx": $pricing,
    "./routes/submit.tsx": $submit,
    "./routes/therapy-progress.tsx": $therapy_progress,
    "./routes/therapy/[therapyId].tsx": $therapy_therapyId_,
    "./routes/therapy/index.tsx": $therapy_index,
    "./routes/user-profile.tsx": $user_profile,
    "./routes/users/[login].tsx": $users_login_,
    "./routes/welcome.tsx": $welcome,
  },
  islands: {
    "./islands/Chart.tsx": $Chart,
    "./islands/ItemsList.tsx": $ItemsList,
    "./islands/TherapyProgress.tsx": $TherapyProgress,
    "./islands/UsersTable.tsx": $UsersTable,
  },
  baseUrl: import.meta.url,
} satisfies Manifest;

export default manifest;

``````


### main.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/main.ts
`relative_path`: main.ts
`format`: Arbitrary Binary Data
`size`: 823   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
/// <reference no-default-lib="true" />
/// <reference lib="dom" />
/// <reference lib="dom.iterable" />
/// <reference lib="dom.asynciterable" />
/// <reference lib="deno.ns" />
/// <reference lib="deno.unstable" />

import { start } from "$fresh/server.ts";
import manifest from "./fresh.gen.ts";
import config from "./fresh.config.ts";
import { isStripeEnabled } from "@/utils/stripe.ts";

console.log(
  isStripeEnabled()
    ? "`STRIPE_SECRET_KEY` environment variable is defined. Stripe is enabled."
    : "`STRIPE_SECRET_KEY` environment variable is not defined. Stripe is disabled.\n" +
      "For more information on how to set up Stripe, see https://github.com/denoland/saaskit#set-up-stripe-optional",
);

await start(manifest, config);

``````


### LICENSE
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/LICENSE
`relative_path`: LICENSE
`format`: Arbitrary Binary Data
`size`: 1070   


``````
MIT License

Copyright 2023 the Deno authors.

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

``````


### dev.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/dev.ts
`relative_path`: dev.ts
`format`: Arbitrary Binary Data
`size`: 253   


``````
#!/usr/bin/env -S deno run -A --watch=static/,routes/
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.

import dev from "$fresh/dev.ts";
import config from "./fresh.config.ts";

await dev(import.meta.url, "./main.ts", config);

``````


### plugins/welcome.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/welcome.ts
`relative_path`: plugins/welcome.ts
`format`: Arbitrary Binary Data
`size`: 590   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type { Plugin } from "$fresh/server.ts";
import { isGitHubSetup } from "@/utils/github.ts";
import { redirect } from "@/utils/http.ts";

export default {
  name: "welcome",
  middlewares: [{
    path: "/",
    middleware: {
      handler: async (req, ctx) => {
        const { pathname } = new URL(req.url);
        return !isGitHubSetup() && pathname !== "/welcome" &&
            ctx.destination === "route"
          ? redirect("/welcome")
          : await ctx.next();
      },
    },
  }],
} as Plugin;

``````


### plugins/error_handling.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/error_handling.ts
`relative_path`: plugins/error_handling.ts
`format`: Arbitrary Binary Data
`size`: 1738   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type { Plugin } from "$fresh/server.ts";
import { STATUS_CODE, STATUS_TEXT } from "$std/http/status.ts";
import type { State } from "@/plugins/session.ts";
import { BadRequestError, redirect, UnauthorizedError } from "@/utils/http.ts";

/**
 * Returns the HTTP status code corresponding to a given runtime error. By
 * default, a HTTP 500 status code is returned.
 *
 * @example
 * ```ts
 * import { toErrorStatus } from "@/plugins/error_handling.ts";
 *
 * toErrorStatus(new Deno.errors.NotFound) // Returns 404
 * ```
 */
export function toErrorStatus(error: Error) {
  if (error instanceof Deno.errors.NotFound) return STATUS_CODE.NotFound;
  if (error instanceof UnauthorizedError) return STATUS_CODE.Unauthorized;
  if (error instanceof BadRequestError) return STATUS_CODE.BadRequest;
  return STATUS_CODE.InternalServerError;
}

export default {
  name: "error-handling",
  middlewares: [
    {
      path: "/",
      middleware: {
        async handler(_req, ctx) {
          try {
            return await ctx.next();
          } catch (error) {
            if (error instanceof UnauthorizedError) {
              return redirect("/login");
            }
            throw error;
          }
        },
      },
    },
    {
      path: "/api",
      middleware: {
        async handler(_req, ctx) {
          try {
            return await ctx.next();
          } catch (error) {
            const status = toErrorStatus(error as Error);
            return new Response((error as Error).message, {
              statusText: STATUS_TEXT[status],
              status,
            });
          }
        },
      },
    },
  ],
} as Plugin<State>;

``````


### plugins/blog/mod.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/blog/mod.ts
`relative_path`: plugins/blog/mod.ts
`format`: Arbitrary Binary Data
`size`: 667   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type { Plugin } from "$fresh/server.ts";
import BlogIndex from "./routes/blog/index.tsx";
import BlogSlug from "./routes/blog/[slug].tsx";
import Feed from "./routes/feed.ts";
import { normalize } from "$std/url/normalize.ts";

export function blog(): Plugin {
  return {
    name: "blog",
    routes: [{
      path: "/blog",
      component: BlogIndex,
    }, {
      path: "/blog/[slug]",
      component: BlogSlug,
    }, {
      path: "/feed",
      component: Feed,
    }],
    location: import.meta.url,
    projectLocation: normalize(import.meta.url + "../../../").href,
  };
}

``````


### plugins/blog/utils/posts_test.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/blog/utils/posts_test.ts
`relative_path`: plugins/blog/utils/posts_test.ts
`format`: Arbitrary Binary Data
`size`: 683   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { getPost, getPosts } from "./posts.ts";

import { assert, assertEquals } from "$std/assert/mod.ts";

Deno.test("[blog] getPost()", async () => {
  const post = await getPost("first-post");
  assert(post);
  assertEquals(post.publishedAt, new Date("2022-11-04T15:00:00.000Z"));
  assertEquals(post.summary, "This is an excerpt of my first blog post.");
  assertEquals(post.title, "This is my first blog post!");
  assertEquals(await getPost("third-post"), null);
});

Deno.test("[blog] getPosts()", async () => {
  const posts = await getPosts();
  assert(posts);
  assertEquals(posts.length, 2);
});

``````


### plugins/blog/utils/posts.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/blog/utils/posts.ts
`relative_path`: plugins/blog/utils/posts.ts
`format`: Arbitrary Binary Data
`size`: 2314   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { extract } from "$std/front_matter/yaml.ts";
import { join } from "$std/path/join.ts";

/**
 * This code is based on the
 * {@link https://deno.com/blog/build-a-blog-with-fresh|How to Build a Blog with Fresh}
 * blog post.
 */

export interface Post {
  slug: string;
  title: string;
  publishedAt: Date;
  content: string;
  summary: string;
}

/**
 * Returns a {@linkcode Post} object of by reading and parsing a file with the
 * given slug in the `./posts` folder. Returns `null` if the given file is
 * not a readable or parsable file.
 *
 * @see {@link https://deno.land/api?s=Deno.readTextFile}
 *
 * @example
 * ```ts
 * import { getPost } from "@/utils/posts.ts";
 *
 * const post = await getPost("first-post")!;
 *
 * post?.title; // Returns "This is my first blog post!"
 * post?.publishedAt; // Returns 2022-11-04T15:00:00.000Z
 * post?.slug; // Returns "This is an excerpt of my first blog post."
 * post?.content; // Returns '# Heading 1\n\nHello, world!\n\n```javascript\nconsole.log("Hello World");\n```\n'
 * ```
 */
export async function getPost(slug: string): Promise<Post | null> {
  try {
    const text = await Deno.readTextFile(join("./posts", `${slug}.md`));
    const { attrs, body } = extract<Post>(text);
    return {
      ...attrs,
      slug,
      content: body,
    };
  } catch {
    return null;
  }
}

/**
 * Returns an array of {@linkcode Post} objects by reading and parsing files
 * in the `./posts` folder.
 *
 * @see {@link https://deno.land/api?s=Deno.readDir}
 *
 * @example
 * ```ts
 * import { getPosts } from "@/utils/posts.ts";
 *
 * const posts = await getPosts();
 *
 * posts[0].title; // Returns "This is my first blog post!"
 * posts[0].publishedAt; // Returns 2022-11-04T15:00:00.000Z
 * posts[0].slug; // Returns "This is an excerpt of my first blog post."
 * posts[0].content; // Returns '# Heading 1\n\nHello, world!\n\n```javascript\nconsole.log("Hello World");\n```\n'
 * ```
 */
export async function getPosts(): Promise<Post[]> {
  const posts = await Array.fromAsync(
    Deno.readDir("./posts"),
    async (file) => await getPost(file.name.replace(".md", "")),
  ) as Post[];
  return posts.toSorted((a, b) =>
    b.publishedAt.getTime() - a.publishedAt.getTime()
  );
}

``````


### plugins/blog/components/Share.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/blog/components/Share.tsx
`relative_path`: plugins/blog/components/Share.tsx
`format`: Arbitrary Binary Data
`size`: 1824   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import IconBrandFacebook from "tabler_icons_tsx/brand-facebook.tsx";
import IconBrandLinkedin from "tabler_icons_tsx/brand-linkedin.tsx";
import IconBrandReddit from "tabler_icons_tsx/brand-reddit.tsx";
import IconBrandTwitter from "tabler_icons_tsx/brand-twitter.tsx";

/**
 * Dynamically generates links for sharing the current content on the major
 * social media platforms.
 *
 * @see {@link https://schier.co/blog/pure-html-share-buttons}
 */
export default function Share(props: { url: URL; title: string }) {
  return (
    <div class="flex flex-row gap-4 my-4">
      <span class="align-middle">Share</span>
      <a
        href={`https://www.facebook.com/sharer/sharer.php?u=${
          encodeURIComponent(props.url.href)
        }`}
        target="_blank"
        aria-label={`Share ${props.title} on Facebook`}
      >
        <IconBrandFacebook />
      </a>
      <a
        href={`https://www.linkedin.com/shareArticle?url=${
          encodeURIComponent(props.url.href)
        }&title=${encodeURIComponent(props.title)}`}
        target="_blank"
        aria-label={`Share ${props.title} on LinkedIn`}
      >
        <IconBrandLinkedin />
      </a>
      <a
        href={`https://reddit.com/submit?url=${
          encodeURIComponent(props.url.href)
        }&title=${encodeURIComponent(props.title)}`}
        target="_blank"
        aria-label={`Share ${props.title} on Reddit`}
      >
        <IconBrandReddit />
      </a>
      <a
        href={`https://twitter.com/share?url=${
          encodeURIComponent(props.url.href)
        }&text=${encodeURIComponent(props.title)}`}
        target="_blank"
        aria-label={`Share ${props.title} on Twitter`}
      >
        <IconBrandTwitter />
      </a>
    </div>
  );
}

``````


### plugins/blog/routes/blog/index.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/blog/routes/blog/index.tsx
`relative_path`: plugins/blog/routes/blog/index.tsx
`format`: Arbitrary Binary Data
`size`: 1221   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { defineRoute } from "$fresh/server.ts";
import { getPosts, type Post } from "../../utils/posts.ts";
import Head from "@/components/Head.tsx";

function PostCard(props: Post) {
  return (
    <div class="py-8">
      <a class="sm:col-span-2" href={`/blog/${props.slug}`}>
        <h2 class="text-2xl font-bold">
          {props.title}
        </h2>
        {props.publishedAt.toString() !== "Invalid Date" && (
          <time
            dateTime={props.publishedAt.toISOString()}
            class="text-gray-500"
          >
            {props.publishedAt.toLocaleDateString("en-US", {
              dateStyle: "long",
            })}
          </time>
        )}
        <div class="mt-4">
          {props.summary}
        </div>
      </a>
    </div>
  );
}

export default defineRoute(async (_req, ctx) => {
  const posts = await getPosts();

  return (
    <>
      <Head title="Blog" href={ctx.url.href} />
      <main class="p-4 flex-1">
        <h1 class="heading-with-margin-styles">Blog</h1>
        <div class="divide-y">
          {posts.map((post) => <PostCard {...post} />)}
        </div>
      </main>
    </>
  );
});

``````


### plugins/blog/routes/blog/[slug].tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/blog/routes/blog/[slug].tsx
`relative_path`: plugins/blog/routes/blog/[slug].tsx
`format`: Arbitrary Binary Data
`size`: 1355   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { defineRoute } from "$fresh/server.ts";
import { CSS, render } from "jsr:@deno/gfm";
import { getPost } from "../../utils/posts.ts";
import Head from "@/components/Head.tsx";
import Share from "../../components/Share.tsx";

export default defineRoute(async (_req, ctx) => {
  const post = await getPost(ctx.params.slug);
  if (post === null) return await ctx.renderNotFound();

  return (
    <>
      <Head title={post.title} href={ctx.url.href}>
        <style dangerouslySetInnerHTML={{ __html: CSS }} />
      </Head>
      <main class="p-4 flex-1">
        <h1 class="text-4xl font-bold">{post.title}</h1>
        {post.publishedAt.toString() !== "Invalid Date" && (
          <time
            dateTime={post.publishedAt.toISOString()}
            class="text-gray-500"
          >
            {post.publishedAt.toLocaleDateString("en-US", {
              dateStyle: "long",
            })}
          </time>
        )}
        <Share url={ctx.url} title={post.title} />
        <div
          class="mt-8 markdown-body !bg-transparent !dark:text-white"
          data-color-mode="auto"
          data-light-theme="light"
          data-dark-theme="dark"
          dangerouslySetInnerHTML={{ __html: render(post.content) }}
        />
      </main>
    </>
  );
});

``````


### plugins/blog/routes/feed.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/blog/routes/feed.ts
`relative_path`: plugins/blog/routes/feed.ts
`format`: Arbitrary Binary Data
`size`: 1370   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { Feed } from "npm:feed@4.2.2";
import { getPosts } from "../utils/posts.ts";
import { SITE_NAME } from "@/utils/constants.ts";
import { defineRoute } from "$fresh/server.ts";

const copyright = `Copyright ${new Date().getFullYear()} ${SITE_NAME}`;

// Use https://validator.w3.org/feed/ to validate RSS feed syntax.
export default defineRoute(async (_req, ctx) => {
  const { origin } = ctx.url;
  const feed = new Feed({
    title: "Deno",
    description: `The latest news from ${SITE_NAME}`,
    id: `${origin}/blog`,
    link: `${origin}/blog`,
    language: "en",
    favicon: `${origin}/favicon.ico`,
    copyright,
    generator: "Feed (https://github.com/jpmonette/feed) for Deno",
    feedLinks: {
      atom: `${origin}/feed`,
    },
  });

  const posts = await getPosts();
  for (const post of posts) {
    feed.addItem({
      id: `${origin}/blog/${post.slug}`,
      title: post.title,
      description: post.summary,
      date: post.publishedAt,
      link: `${origin}/blog/${post.slug}`,
      author: [{ name: "The Deno Authors" }],
      copyright,
      published: new Date(post.publishedAt),
    });
  }

  const atomFeed = feed.atom1();
  return new Response(atomFeed, {
    headers: {
      "content-type": "application/atom+xml; charset=utf-8",
    },
  });
});

``````


### plugins/session.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/session.ts
`relative_path`: plugins/session.ts
`format`: Arbitrary Binary Data
`size`: 2396   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { Plugin } from "$fresh/server.ts";
import type { FreshContext } from "$fresh/server.ts";
import { getSessionId } from "kv_oauth/mod.ts";
import { getUserBySession } from "@/utils/db.ts";
import type { User } from "@/utils/db.ts";
import { UnauthorizedError } from "@/utils/http.ts";

export interface State {
  sessionUser?: User;
}

export type SignedInState = Required<State>;

export function assertSignedIn(
  ctx: { state: State },
): asserts ctx is { state: SignedInState } {
  if (ctx.state.sessionUser === undefined) {
    throw new UnauthorizedError("User must be signed in");
  }
}

async function setSessionState(
  req: Request,
  ctx: FreshContext<State>,
) {
  if (ctx.destination !== "route") return await ctx.next();

  // Initial state
  ctx.state.sessionUser = undefined;

  const sessionId = getSessionId(req);
  if (sessionId === undefined) return await ctx.next();
  const user = await getUserBySession(sessionId);
  if (user === null) return await ctx.next();

  ctx.state.sessionUser = user;

  return await ctx.next();
}

async function ensureSignedIn(
  _req: Request,
  ctx: FreshContext<State>,
) {
  assertSignedIn(ctx);
  return await ctx.next();
}

/**
 * Adds middleware to the defined routes that ensures the client is signed-in
 * before proceeding. The {@linkcode ensureSignedIn} middleware throws an error
 * equivalent to the
 * {@link https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401|HTTP 401 Unauthorized}
 * error if `ctx.state.sessionUser` is `undefined`.
 *
 * The thrown error is then handled by {@linkcode handleWebPageErrors}, or
 * {@linkcode handleRestApiErrors}, if the request is made to a REST API
 * endpoint.
 *
 * @see {@link https://fresh.deno.dev/docs/concepts/plugins|Plugins documentation}
 * for more information on Fresh's plugin functionality.
 */
export default {
  name: "session",
  middlewares: [
    {
      path: "/",
      middleware: { handler: setSessionState },
    },
    {
      path: "/account",
      middleware: { handler: ensureSignedIn },
    },
    {
      path: "/dashboard",
      middleware: { handler: ensureSignedIn },
    },
    {
      path: "/api/me",
      middleware: { handler: ensureSignedIn },
    },
    {
      path: "/api/vote",
      middleware: { handler: ensureSignedIn },
    },
  ],
} as Plugin<State>;

``````


### plugins/security_headers.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/security_headers.ts
`relative_path`: plugins/security_headers.ts
`format`: Arbitrary Binary Data
`size`: 1146   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type { Plugin } from "$fresh/server.ts";

export default {
  name: "security-headers",
  middlewares: [
    {
      path: "/",
      middleware: {
        handler: async (req, ctx) => {
          if (
            ctx.destination !== "route" ||
            new URL(req.url).pathname.startsWith("/api")
          ) return await ctx.next();

          const response = await ctx.next();

          /**
           * @todo(Jabolol) Implement `Content-Security-Policy` once
           * https://github.com/denoland/fresh/pull/1787 lands.
           */
          response.headers.set(
            "Strict-Transport-Security",
            "max-age=63072000;",
          );
          response.headers.set(
            "Referrer-Policy",
            "strict-origin-when-cross-origin",
          );
          response.headers.set("X-Content-Type-Options", "nosniff");
          response.headers.set("X-Frame-Options", "SAMEORIGIN");
          response.headers.set("X-XSS-Protection", "1; mode=block");

          return response;
        },
      },
    },
  ],
} as Plugin;

``````


### plugins/kv_oauth.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/plugins/kv_oauth.ts
`relative_path`: plugins/kv_oauth.ts
`format`: Arbitrary Binary Data
`size`: 3696   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type { Plugin } from "$fresh/server.ts";
import {
  createGitHubOAuthConfig,
  createGoogleOAuthConfig,
  handleCallback,
  signIn,
  signOut,
} from "kv_oauth/mod.ts";
import {
  createUser,
  getUser,
  updateUserSession,
  type User,
} from "@/utils/db.ts";
import { isStripeEnabled, stripe } from "@/utils/stripe.ts";
import { getGitHubUser } from "@/utils/github.ts";

// Exported for mocking and spying in e2e tests
export const _internals = { handleCallback };

/**
 * This custom plugin centralizes all authentication logic using the
 * {@link https://deno.land/x/deno_kv_oauth|Deno KV OAuth} module.
 *
 * The implementation is based off Deno KV OAuth's own
 * {@link https://deno.land/x/deno_kv_oauth/src/fresh_plugin.ts?source|Fresh plugin}
 * implementation.
 */
export default {
  name: "kv-oauth",
  routes: [
    {
      path: "/auth/github",
      handler: async (req) => await signIn(req, createGitHubOAuthConfig()),
    },
    {
      path: "/callback",
      handler: async (req) => {
        const { response, tokens, sessionId } = await _internals.handleCallback(
          req,
          createGitHubOAuthConfig(),
        );

        const githubUser = await getGitHubUser(tokens.accessToken);
        const user = await getUser(githubUser.login);

        if (user === null) {
          const newUser: User = {
            login: githubUser.login,
            sessionId,
            isSubscribed: false,
          };
          if (isStripeEnabled()) {
            const customer = await stripe.customers.create();
            newUser.stripeCustomerId = customer.id;
          }
          await createUser(newUser);
        } else {
          await updateUserSession(user, sessionId);
        }

        return response;
      },
    },
    {
      path: "/auth/google",
      handler: async (req) =>
        await signIn(
          req,
          createGoogleOAuthConfig({
            redirectUri:
              "https://vacuul-platform.deno.dev/auth/google/callback",
            scope:
              "https://www.googleapis.com/auth/userinfo.profile https://www.googleapis.com/auth/userinfo.email",
          }),
        ),
    },
    {
      path: "/auth/google/callback",
      handler: async (req) => {
        const { response, tokens, sessionId } = await _internals.handleCallback(
          req,
          createGoogleOAuthConfig({
            redirectUri:
              "https://vacuul-platform.deno.dev/auth/google/callback",
            scope:
              "https://www.googleapis.com/auth/userinfo.profile https://www.googleapis.com/auth/userinfo.email",
          }),
        );

        const googleUser = await fetch(
          "https://www.googleapis.com/oauth2/v3/userinfo",
          {
            headers: {
              Authorization: `Bearer ${tokens.accessToken}`,
            },
          },
        ).then((res) => res.json());

        const user = await getUser(googleUser.email || googleUser.sub);

        if (user === null) {
          const newUser: User = {
            login: googleUser.email || googleUser.sub,
            sessionId,
            isSubscribed: false,
            picture: googleUser.picture,
            name: googleUser.name,
          };

          if (isStripeEnabled()) {
            const customer = await stripe.customers.create();
            newUser.stripeCustomerId = customer.id;
          }
          await createUser(newUser);
        } else {
          await updateUserSession(user, sessionId);
        }

        return response;
      },
    },
    {
      path: "/signout",
      handler: signOut,
    },
  ],
} as Plugin;

``````


### Dockerfile
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/Dockerfile
`relative_path`: Dockerfile
`format`: Arbitrary Binary Data
`size`: 149   


``````
FROM denoland/deno
EXPOSE 8000
WORKDIR /app
ADD . /app

# Add dependencies to the container's Deno cache
RUN deno cache main.ts
CMD ["task", "start"]
``````


### deno.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/deno.json
`relative_path`: deno.json
`format`: Arbitrary Binary Data
`size`: 2441   


``````
{
  "lock": false,
  "tasks": {
    "init:stripe": "deno run --allow-read --allow-env --allow-net --env tasks/init_stripe.ts",
    "db:dump": "deno run --allow-read --allow-env --unstable-kv tasks/db_dump.ts",
    "db:restore": "deno run --allow-read --allow-env --unstable-kv tasks/db_restore.ts",
    "db:seed": "deno run --allow-read --allow-env --allow-net --unstable-kv tasks/db_seed.ts",
    "db:migrate": "deno run --allow-read --allow-env --allow-net --unstable-kv tasks/db_migrate.ts",
    "db:reset": "deno run --allow-read --allow-env --unstable-kv tasks/db_reset.ts",
    "start": "deno run --unstable-kv -A --watch=static/,routes/ --env dev.ts",
    "test": "DENO_KV_PATH=:memory: deno test -A --parallel --unstable-kv --coverage",
    "check:license": "deno run --allow-read --allow-write tasks/check_license.ts",
    "check:types": "deno check main.ts && deno check dev.ts && deno check tasks/*.ts",
    "ok": "deno fmt --check && deno lint && deno task check:license --check && deno task check:types && deno task test",
    "cov:gen": "deno coverage coverage --lcov --exclude='.tsx' --output=cov.lcov",
    "update": "deno run -A -r https://fresh.deno.dev/update .",
    "build": "deno run -A --unstable-kv dev.ts build",
    "preview": "deno run -A --unstable-kv main.ts"
  },
  "compilerOptions": { "jsx": "react-jsx", "jsxImportSource": "preact" },
  "imports": {
    "@/": "./",
    "$fresh/": "https://raw.githubusercontent.com/denoland/fresh/60220dd33b5b0f6b5c72927c933dbc32a3c4734e/",
    "preact": "https://esm.sh/preact@10.19.2",
    "preact/": "https://esm.sh/preact@10.19.2/",
    "preact-render-to-string": "https://esm.sh/*preact-render-to-string@6.2.2",
    "@preact/signals": "https://esm.sh/*@preact/signals@1.2.1",
    "@preact/signals-core": "https://esm.sh/*@preact/signals-core@1.5.0",
    "tailwindcss": "npm:tailwindcss@3.4.1",
    "tailwindcss/": "npm:/tailwindcss@3.4.1/",
    "tailwindcss/plugin": "npm:/tailwindcss@3.4.1/plugin.js",
    "$std/": "https://deno.land/std@0.208.0/",
    "stripe": "npm:/stripe@13.5.0",
    "kv_oauth/": "https://deno.land/x/deno_kv_oauth@v0.9.1/",
    "tabler_icons_tsx/": "https://deno.land/x/tabler_icons_tsx@0.0.4/tsx/",
    "fresh_charts/": "https://deno.land/x/fresh_charts@0.3.1/",
    "lucide-preact": "https://esm.sh/lucide-preact@0.456.0"
  },
  "exclude": ["coverage/", "_fresh/", "**/_fresh/*"],
  "lint": { "rules": { "tags": ["fresh", "recommended"] } }
}

``````


### islands/Chart.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/islands/Chart.tsx
`relative_path`: islands/Chart.tsx
`format`: Arbitrary Binary Data
`size`: 135   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
export { Chart as default } from "fresh_charts/island.tsx";

``````


### islands/UsersTable.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/islands/UsersTable.tsx
`relative_path`: islands/UsersTable.tsx
`format`: Arbitrary Binary Data
`size`: 2783   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { useSignal } from "@preact/signals";
import { useEffect } from "preact/hooks";
import type { User } from "@/utils/db.ts";
import GitHubAvatarImg from "@/components/GitHubAvatarImg.tsx";
import { fetchValues } from "@/utils/http.ts";
import { PremiumBadge } from "@/components/PremiumBadge.tsx";

const TH_STYLES = "p-4 text-left";
const TD_STYLES = "p-4";

function UserTableRow(props: User) {
  return (
    <tr class="hover:bg-gray-50 hover:dark:bg-gray-900 border-b border-gray-200">
      <td scope="col" class={TD_STYLES}>
        <GitHubAvatarImg login={props.login} size={32} />
        <a
          class="hover:underline ml-4 align-middle"
          href={"/users/" + props.login}
        >
          {props.login}
        </a>
      </td>
      <td scope="col" class={TD_STYLES + " text-gray-500"}>
        {props.isSubscribed
          ? (
            <>
              Premium <PremiumBadge class="size-5 inline" />
            </>
          )
          : "Basic"}
      </td>
      <td scope="col" class={TD_STYLES + " text-gray-500"}>
        ${(Math.random() * 100).toFixed(2)}
      </td>
    </tr>
  );
}

export interface UsersTableProps {
  /** Endpoint URL of the REST API to make the fetch request to */
  endpoint: string;
}

export default function UsersTable(props: UsersTableProps) {
  const usersSig = useSignal<User[]>([]);
  const cursorSig = useSignal("");
  const isLoadingSig = useSignal(false);

  async function loadMoreUsers() {
    if (isLoadingSig.value) return;
    isLoadingSig.value = true;
    try {
      const { values, cursor } = await fetchValues<User>(
        props.endpoint,
        cursorSig.value,
      );
      usersSig.value = [...usersSig.value, ...values];
      cursorSig.value = cursor;
    } catch (error) {
      console.log((error as Error).message);
    } finally {
      isLoadingSig.value = false;
    }
  }

  useEffect(() => {
    loadMoreUsers();
  }, []);

  return (
    <div class="w-full rounded-lg shadow border-1 border-gray-300 overflow-x-auto">
      <table class="table-auto border-collapse w-full">
        <thead class="border-b border-gray-300">
          <tr>
            <th scope="col" class={TH_STYLES}>User</th>
            <th scope="col" class={TH_STYLES}>Subscription</th>
            <th scope="col" class={TH_STYLES}>Revenue</th>
          </tr>
        </thead>
        <tbody>
          {usersSig.value.map((user) => <UserTableRow {...user} />)}
        </tbody>
      </table>
      {cursorSig.value !== "" && (
        <button
          onClick={loadMoreUsers}
          class="link-styles p-4"
        >
          {isLoadingSig.value ? "Loading..." : "Load more"}
        </button>
      )}
    </div>
  );
}

``````


### islands/ItemsList.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/islands/ItemsList.tsx
`relative_path`: islands/ItemsList.tsx
`format`: Arbitrary Binary Data
`size`: 5321   




### islands/TherapyProgress.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/islands/TherapyProgress.tsx
`relative_path`: islands/TherapyProgress.tsx
`format`: Arbitrary Binary Data
`size`: 5140   




### utils/stripe.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/utils/stripe.ts
`relative_path`: utils/stripe.ts
`format`: Arbitrary Binary Data
`size`: 1265   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import Stripe from "stripe";
import { AssertionError } from "$std/assert/assertion_error.ts";

const STRIPE_SECRET_KEY = Deno.env.get("STRIPE_SECRET_KEY");

export function isStripeEnabled() {
  return Deno.env.has("STRIPE_SECRET_KEY");
}

export function getStripePremiumPlanPriceId() {
  return Deno.env.get(
    "STRIPE_PREMIUM_PLAN_PRICE_ID",
  );
}

export const stripe = new Stripe(STRIPE_SECRET_KEY!, {
  apiVersion: "2023-08-16",
  // Use the Fetch API instead of Node's HTTP client.
  httpClient: Stripe.createFetchHttpClient(),
});

/**
 * Asserts that the value is strictly a {@linkcode Stripe.Price} object.
 *
 * @example
 * ```ts
 * import { assertIsPrice } from "@/utils/stripe.ts";
 *
 * assertIsPrice(undefined); // Throws AssertionError
 * assertIsPrice(null); // Throws AssertionError
 * assertIsPrice("not a price"); // Throws AssertionError
 * ```
 */
export function assertIsPrice(value: unknown): asserts value is Stripe.Price {
  if (value === undefined || value === null || typeof value === "string") {
    throw new AssertionError(
      "Default price must be of type `Stripe.Price`. Please run the `deno task init:stripe` as the README instructs.",
    );
  }
}

``````


### utils/github.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/utils/github.ts
`relative_path`: utils/github.ts
`format`: Arbitrary Binary Data
`size`: 1145   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { createGitHubOAuthConfig } from "kv_oauth/mod.ts";
import { BadRequestError } from "@/utils/http.ts";

export function isGitHubSetup() {
  try {
    createGitHubOAuthConfig();
    return true;
  } catch {
    return false;
  }
}

interface GitHubUser {
  login: string;
  email: string;
}

/**
 * Returns the GitHub profile information of the user with the given access
 * token.
 *
 * @see {@link https://docs.github.com/en/rest/users/users?apiVersion=2022-11-28#get-the-authenticated-user}
 *
 * @example
 * ```ts
 * import { getGitHubUser } from "@/utils/github.ts";
 *
 * const user = await getGitHubUser("<access token>");
 * user.login; // Returns "octocat"
 * user.email; // Returns "octocat@github.com"
 * ```
 */
export async function getGitHubUser(accessToken: string) {
  const resp = await fetch("https://api.github.com/user", {
    headers: { authorization: `Bearer ${accessToken}` },
  });
  if (!resp.ok) {
    const { message } = await resp.json();
    throw new BadRequestError(message);
  }
  return await resp.json() as Promise<GitHubUser>;
}

``````


### utils/display_test.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/utils/display_test.ts
`relative_path`: utils/display_test.ts
`format`: Arbitrary Binary Data
`size`: 1551   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { formatCurrency, pluralize, timeAgo } from "./display.ts";
import { DAY, HOUR, MINUTE, SECOND } from "$std/datetime/constants.ts";
import { assertEquals, assertThrows } from "$std/assert/mod.ts";

Deno.test("[display] pluralize()", () => {
  assertEquals(pluralize(0, "item"), "0 items");
  assertEquals(pluralize(1, "item"), "1 item");
  assertEquals(pluralize(2, "item"), "2 items");
});

Deno.test("[display] timeAgo()", () => {
  assertEquals(timeAgo(new Date(Date.now())), "just now");
  assertEquals(timeAgo(new Date(Date.now() - SECOND * 30)), "30 seconds ago");
  assertEquals(timeAgo(new Date(Date.now() - MINUTE)), "1 minute ago");
  assertEquals(timeAgo(new Date(Date.now() - MINUTE * 2)), "2 minutes ago");
  assertEquals(timeAgo(new Date(Date.now() - MINUTE * 59)), "59 minutes ago");
  assertEquals(timeAgo(new Date(Date.now() - HOUR)), "1 hour ago");
  assertEquals(
    timeAgo(new Date(Date.now() - HOUR - MINUTE * 35)),
    "1 hour ago",
  );
  assertEquals(timeAgo(new Date(Date.now() - HOUR * 2)), "2 hours ago");
  assertEquals(timeAgo(new Date(Date.now() - DAY)), "1 day ago");
  assertEquals(timeAgo(new Date(Date.now() - DAY - HOUR * 12)), "1 day ago");
  assertEquals(timeAgo(new Date(Date.now() - DAY * 5)), "5 days ago");
  assertThrows(
    () => timeAgo(new Date(Date.now() + 1)),
    Error,
    "Timestamp must be in the past",
  );
});

Deno.test("[display] formatCurrency()", () => {
  assertEquals(formatCurrency(5, "USD"), "$5");
});

``````


### utils/http.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/utils/http.ts
`relative_path`: utils/http.ts
`format`: Arbitrary Binary Data
`size`: 2435   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { RedirectStatus, STATUS_CODE } from "$std/http/status.ts";

/**
 * Returns a response that redirects the client to the given location (URL).
 *
 * @param location A relative (to the request URL) or absolute URL.
 * @param status HTTP status
 *
 * @see {@link https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Location}
 *
 * @example
 * ```ts
 * import { redirect } from "@/utils/http.ts";
 *
 * redirect("/new-page"); // Redirects client to `/new-page` with HTTP status 303
 * redirect("/new-page", 301); // Redirects client to `/new-page` with HTTP status 301
 * ```
 */
export function redirect(
  location: string,
  status: typeof STATUS_CODE.Created | RedirectStatus = STATUS_CODE.SeeOther,
) {
  return new Response(null, {
    headers: {
      location,
    },
    status,
  });
}

/**
 * Returns the `cursor` URL parameter value of the given URL.
 *
 * @example
 * ```ts
 * import { getCursor } from "@/utils/http.ts";
 *
 * getCursor(new URL("http://example.com?cursor=12345")); // Returns "12345"
 * getCursor(new URL("http://example.com")); // Returns ""
 * ```
 */
export function getCursor(url: URL) {
  return url.searchParams.get("cursor") ?? "";
}

/**
 * Returns the values and cursor for the resource of a given endpoint. In the
 * backend, the request handler collects these values and cursor by iterating
 * through a {@linkcode Deno.KvListIterator}
 *
 * @example
 * ```ts
 * import { fetchValues } from "@/utils/http.ts";
 * import type { Item } from "@/utils/db.ts";
 *
 * const body = await fetchValues<Item>("https://hunt.deno.land/api/items", "12345");
 * body.values[0].id; // Returns "13f34b7e-5563-4001-98ed-9ee04d7af717"
 * body.values[0].url; // Returns "http://example.com"
 * body.cursor; // Returns "12346"
 * ```
 */
export async function fetchValues<T>(endpoint: string, cursor: string) {
  let url = endpoint;
  if (cursor !== "") url += "?cursor=" + cursor;
  const resp = await fetch(url);
  if (!resp.ok) throw new Error(`Request failed: GET ${url}`);
  return await resp.json() as { values: T[]; cursor: string };
}

export class UnauthorizedError extends Error {
  constructor(message?: string) {
    super(message);
    this.name = "UnauthorizedError";
  }
}

export class BadRequestError extends Error {
  constructor(message?: string) {
    super(message);
    this.name = "BadRequestError";
  }
}

``````


### utils/db_test.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/utils/db_test.ts
`relative_path`: utils/db_test.ts
`format`: Arbitrary Binary Data
`size`: 4053   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { assertEquals, assertRejects } from "$std/assert/mod.ts";
import { ulid } from "$std/ulid/mod.ts";
import {
  collectValues,
  createItem,
  createUser,
  createVote,
  getAreVotedByUser,
  getItem,
  getUser,
  getUserBySession,
  getUserByStripeCustomer,
  type Item,
  listItems,
  listItemsByUser,
  listItemsVotedByUser,
  randomItem,
  randomUser,
  updateUser,
  updateUserSession,
  type User,
} from "./db.ts";

Deno.test("[db] items", async () => {
  const user = randomUser();
  const item1: Item = {
    ...randomItem(),
    id: ulid(),
    userLogin: user.login,
  };
  const item2: Item = {
    ...randomItem(),
    id: ulid(Date.now() + 1_000),
    userLogin: user.login,
  };

  assertEquals(await getItem(item1.id), null);
  assertEquals(await getItem(item2.id), null);
  assertEquals(await collectValues(listItems()), []);
  assertEquals(await collectValues(listItemsByUser(user.login)), []);

  await createItem(item1);
  await createItem(item2);
  await assertRejects(async () => await createItem(item1));

  assertEquals(await getItem(item1.id), item1);
  assertEquals(await getItem(item2.id), item2);
  assertEquals(await collectValues(listItems()), [item1, item2]);
  assertEquals(await collectValues(listItemsByUser(user.login)), [
    item1,
    item2,
  ]);
});

Deno.test("[db] user", async () => {
  const user = randomUser();

  assertEquals(await getUser(user.login), null);
  assertEquals(await getUserBySession(user.sessionId), null);
  assertEquals(await getUserByStripeCustomer(user.stripeCustomerId!), null);

  await createUser(user);
  await assertRejects(async () => await createUser(user));
  assertEquals(await getUser(user.login), user);
  assertEquals(await getUserBySession(user.sessionId), user);
  assertEquals(await getUserByStripeCustomer(user.stripeCustomerId!), user);

  const subscribedUser: User = { ...user, isSubscribed: true };
  await updateUser(subscribedUser);
  assertEquals(await getUser(subscribedUser.login), subscribedUser);
  assertEquals(
    await getUserBySession(subscribedUser.sessionId),
    subscribedUser,
  );
  assertEquals(
    await getUserByStripeCustomer(subscribedUser.stripeCustomerId!),
    subscribedUser,
  );

  const newSessionId = crypto.randomUUID();
  await updateUserSession(user, newSessionId);
  assertEquals(await getUserBySession(user.sessionId), null);
  assertEquals(await getUserBySession(newSessionId), {
    ...user,
    sessionId: newSessionId,
  });

  await assertRejects(
    async () => await updateUserSession(user, newSessionId),
    Error,
    "Failed to update user session",
  );
});

Deno.test("[db] votes", async () => {
  const item = randomItem();
  const user = randomUser();
  const vote = {
    itemId: item.id,
    userLogin: user.login,
    createdAt: new Date(),
  };

  assertEquals(await collectValues(listItemsVotedByUser(user.login)), []);

  await assertRejects(
    async () => await createVote(vote),
    Deno.errors.NotFound,
    "Item not found",
  );
  await createItem(item);
  await assertRejects(
    async () => await createVote(vote),
    Deno.errors.NotFound,
    "User not found",
  );
  await createUser(user);
  await createVote(vote);
  item.score++;

  assertEquals(await collectValues(listItemsVotedByUser(user.login)), [item]);
  await assertRejects(async () => await createVote(vote));
});

Deno.test("[db] getAreVotedByUser()", async () => {
  const item = randomItem();
  const user = randomUser();
  const vote = {
    itemId: item.id,
    userLogin: user.login,
    createdAt: new Date(),
  };

  assertEquals(await getItem(item.id), null);
  assertEquals(await getUser(user.login), null);
  assertEquals(await getAreVotedByUser([item], user.login), [false]);

  await createItem(item);
  await createUser(user);
  await createVote(vote);
  item.score++;

  assertEquals(await getItem(item.id), item);
  assertEquals(await getUser(user.login), user);
  assertEquals(await getAreVotedByUser([item], user.login), [true]);
});

``````


### utils/display.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/utils/display.ts
`relative_path`: utils/display.ts
`format`: Arbitrary Binary Data
`size`: 2214   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { difference } from "$std/datetime/difference.ts";

/**
 * Returns a pluralized string for the given amount and unit.
 *
 * @example
 * ```ts
 * import { pluralize } from "@/utils/display.ts";
 *
 * pluralize(0, "meow"); // Returns "0 meows"
 * pluralize(1, "meow"); // Returns "1 meow"
 * ```
 */
export function pluralize(amount: number, unit: string) {
  return amount === 1 ? `${amount} ${unit}` : `${amount} ${unit}s`;
}

/**
 * Returns how long ago a given date is from now.
 *
 * @example
 * ```ts
 * import { timeAgo } from "@/utils/display.ts";
 * import { SECOND, MINUTE, HOUR } from "$std/datetime/constants.ts";
 *
 * timeAgo(new Date()); // Returns "just now"
 * timeAgo(new Date(Date.now() - 3 * HOUR)); // Returns "3 hours ago"
 * ```
 */
export function timeAgo(date: Date) {
  const now = new Date();
  if (date > now) throw new Error("Timestamp must be in the past");
  const match = Object.entries(
    difference(now, date, {
      // These units make sense for a web UI
      units: [
        "seconds",
        "minutes",
        "hours",
        "days",
        "weeks",
        "months",
        "years",
      ],
    }),
  )
    .toReversed()
    .find(([_, amount]) => amount > 0);
  if (match === undefined) return "just now";
  const [unit, amount] = match;
  // Remove the last character which is an "s"
  return pluralize(amount, unit.slice(0, -1)) + " ago";
}

/**
 * Returns a formatted string based on the given amount of currency and the
 * `en-US` locale. Change the locale for your use case as required.
 *
 * @see {@linkcode Intl.NumberFormat}
 *
 * @example
 * ```ts
 * import { formatCurrency } from "@/utils/display.ts";
 *
 * formatCurrency(5, "USD"); // Returns "$5"
 * ```
 */
export function formatCurrency(
  amount: number,
  currency: string,
): string {
  return new Intl.NumberFormat(
    "en-US",
    {
      style: "currency",
      currency,
      currencyDisplay: "symbol",
      maximumFractionDigits: 0,
    },
  ).format(amount)
    // Issue: https://stackoverflow.com/questions/44533919/space-after-symbol-with-js-intl
    .replace(/^(\D+)/, "$1")
    .replace(/\s+/, "");
}

``````


### utils/constants.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/utils/constants.ts
`relative_path`: utils/constants.ts
`format`: Arbitrary Binary Data
`size`: 173   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
export const SITE_NAME = "Vacuul";
export const SITE_DESCRIPTION = "A modern way to do therapy.";

``````


### utils/github_test.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/utils/github_test.ts
`relative_path`: utils/github_test.ts
`format`: Arbitrary Binary Data
`size`: 1306   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { assertRejects } from "$std/assert/assert_rejects.ts";
import { getGitHubUser } from "./github.ts";
import { returnsNext, stub } from "$std/testing/mock.ts";
import { assertEquals } from "$std/assert/assert_equals.ts";
import { STATUS_CODE } from "$std/http/status.ts";
import { BadRequestError } from "@/utils/http.ts";

Deno.test("[plugins] getGitHubUser()", async (test) => {
  await test.step("rejects on error message", async () => {
    const message = crypto.randomUUID();
    const fetchStub = stub(
      globalThis,
      "fetch",
      returnsNext([
        Promise.resolve(
          Response.json({ message }, { status: STATUS_CODE.BadRequest }),
        ),
      ]),
    );
    await assertRejects(
      async () => await getGitHubUser(crypto.randomUUID()),
      BadRequestError,
      message,
    );
    fetchStub.restore();
  });

  await test.step("resolves to a GitHub user object", async () => {
    const body = { login: crypto.randomUUID(), email: crypto.randomUUID() };
    const fetchStub = stub(
      globalThis,
      "fetch",
      returnsNext([Promise.resolve(Response.json(body))]),
    );
    assertEquals(await getGitHubUser(crypto.randomUUID()), body);
    fetchStub.restore();
  });
});

``````


### utils/db.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/utils/db.ts
`relative_path`: utils/db.ts
`format`: Arbitrary Binary Data
`size`: 13166   




### utils/stripe_test.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/utils/stripe_test.ts
`relative_path`: utils/stripe_test.ts
`format`: Arbitrary Binary Data
`size`: 591   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { AssertionError, assertThrows } from "$std/assert/mod.ts";
import { assertIsPrice } from "./stripe.ts";

Deno.test("[stripe] assertIsPrice()", () => {
  const message =
    "Default price must be of type `Stripe.Price`. Please run the `deno task init:stripe` as the README instructs.";
  assertThrows(() => assertIsPrice(undefined), AssertionError, message);
  assertThrows(() => assertIsPrice(null), AssertionError, message);
  assertThrows(() => assertIsPrice("not a price"), AssertionError, message);
});

``````


### utils/http_test.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/utils/http_test.ts
`relative_path`: utils/http_test.ts
`format`: Arbitrary Binary Data
`size`: 1852   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { returnsNext, stub } from "$std/testing/mock.ts";
import { fetchValues, getCursor, redirect } from "./http.ts";
import { assert, assertEquals, assertRejects } from "$std/assert/mod.ts";
import { STATUS_CODE } from "$std/http/status.ts";
import { Item, randomItem } from "@/utils/db.ts";

Deno.test("[http] redirect() defaults", () => {
  const location = "/hello-there";

  const resp = redirect(location);
  assert(!resp.ok);
  assertEquals(resp.body, null);
  assertEquals(resp.headers.get("location"), location);
  assertEquals(resp.status, 303);
});

Deno.test("[http] redirect()", () => {
  const location = "/hello-there";
  const status = 302;

  const resp = redirect(location, status);
  assert(!resp.ok);
  assertEquals(resp.body, null);
  assertEquals(resp.headers.get("location"), location);
  assertEquals(resp.status, status);
});

Deno.test("[http] getCursor()", () => {
  assertEquals(getCursor(new URL("http://example.com")), "");
  assertEquals(getCursor(new URL("http://example.com?cursor=here")), "here");
});

Deno.test("[http] fetchValues()", async () => {
  const resp1 = Promise.resolve(
    new Response(null, { status: STATUS_CODE.NotFound }),
  );
  const resp2Body = {
    values: [randomItem(), randomItem()],
    cursor: crypto.randomUUID(),
  };
  const resp2Cursor = crypto.randomUUID();
  const resp2 = Promise.resolve(Response.json(resp2Body));
  const fetchStub = stub(
    globalThis,
    "fetch",
    returnsNext([resp1, resp2]),
  );
  const endpoint = "http://localhost";
  await assertRejects(
    async () => await fetchValues(endpoint, ""),
    Error,
    `Request failed: GET ${endpoint}`,
  );
  assertEquals(
    await fetchValues<Item>(endpoint + "/api/items", resp2Cursor),
    resp2Body,
  );

  fetchStub.restore();
});

``````


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 15644   




### tailwind.config.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/tailwind.config.ts
`relative_path`: tailwind.config.ts
`format`: Arbitrary Binary Data
`size`: 2329   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { type Config } from "tailwindcss";

export default {
  content: ["{routes,islands,components}/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        primary: "#be185d",
        secondary: "#4338ca",
        accent: "#6C63FF",
        neutral: "#BFBFC4",
        "base-100": "#E5E5E5",
        info: "#3ABFF8",
        success: "#36D399",
        warning: "#FBBD23",
        error: "#F87272",
      },
      fontFamily: {
        sans: ["Inter", "sans-serif"],
        serif: ["Merriweather", "serif"],
      },
      fontSize: {
        "2xs": "0.625rem",
        "3xs": "0.5rem",
        xs: "0.75rem",
        sm: "0.875rem",
        base: "1rem",
        lg: "1.125rem",
        xl: "1.25rem",
        "2xl": "1.5rem",
        "3xl": "1.875rem",
        "4xl": "2.25rem",
        "5xl": "3rem",
        "6xl": "3.75rem",
        "7xl": "4.5rem",
        "8xl": "6rem",
        "9xl": "8rem",
      },
      spacing: {
        18: "4.5rem",
        26: "6.5rem",
        36: "9rem",
        44: "11rem",
        52: "13rem",
        60: "15rem",
        72: "18rem",
        84: "21rem",
        96: "24rem",
      },
      borderWidth: {
        "1": "1px",
        "3": "3px",
        "5": "5px",
        "6": "6px",
      },
      boxShadow: {
        "sm-light": "0 1px 2px rgba(0, 0, 0, 0.05)",
        "md-light": "0 4px 6px rgba(0, 0, 0, 0.1)",
        "lg-light": "0 10px 15px rgba(0, 0, 0, 0.1)",
        "xl-light": "0 20px 25px rgba(0, 0, 0, 0.1)",
        "2xl-light": "0 25px 50px rgba(0, 0, 0, 0.25)",
      },
      animation: {
        fadeIn: "fadeIn 1s ease-in-out forwards",
        slideIn: "slideIn 0.5s ease-out forwards",
        pulse: "pulse 2s infinite",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        slideIn: {
          "0%": { transform: "translateX(-100%)" },
          "100%": { transform: "translateX(0)" },
        },
        pulse: {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: ".5" },
        },
      },
      maxWidth: {
        "8xl": "90rem",
        "9xl": "100rem",
      },
      minHeight: {
        "screen-75": "75vh",
      },
    },
  },
  plugins: [],
} satisfies Config;

``````


### components/Hero.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/components/Hero.tsx
`relative_path`: components/Hero.tsx
`format`: Arbitrary Binary Data
`size`: 1364   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.

export default function Hero() {
  return (
    <section class="py-20 md:py-32">
      <div class="container mx-auto px-6 text-center md:px-12">
        <div class="mx-auto max-w-3xl">
          <h1 class="text-4xl md:text-6xl font-semibold mb-4 text-primary animate-fade-in">
            Bienvenido a <span class="text-primary">Vacuul Platform</span>
          </h1>
          <p class="text-lg md:text-xl text-neutral mb-8 animate-fade-in delay-200">
            test
          </p>
          <a
            href="/start"
            class="inline-block bg-primary hover:bg-primary-dark text-white font-semibold py-3 px-6 rounded-full transition-all duration-300 shadow-md hover:shadow-lg animate-fade-in delay-400"
          >
            test
          </a>
        </div>
      </div>
      <style>
        {`
              .animate-fade-in {
                animation: fadeIn 1.2s ease-in-out forwards;
                opacity: 0;
              }
              @keyframes fadeIn {
                to {
                  opacity: 1;
                }
              }
              .delay-200 {
                animation-delay: 0.2s;
              }
              .delay-400 {
                animation-delay: 0.4s;
              }
            `}
      </style>
    </section>
  );
}

``````


### components/TherapyCard.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/components/TherapyCard.tsx
`relative_path`: components/TherapyCard.tsx
`format`: Arbitrary Binary Data
`size`: 1951   


``````
import { TherapyOption } from "@/types/therapy.ts";
import { Thermometer, Clock } from "lucide-preact";

interface TherapyCardProps {
  therapy: TherapyOption;
  onSelect: () => void;
  selected?: boolean;
}

export const TherapyCard = ({
  therapy,
  onSelect,
  selected = false,
}: TherapyCardProps) => {
  return (
    <div
      className={`relative p-6 rounded-xl shadow-lg transition-all flex flex-col justify-between ${
        selected
          ? "bg-indigo-50 border-2 border-indigo-500"
          : "bg-white hover:shadow-xl"
      }`}
    >
      <div className="flex items-start justify-between">
        <div>
          <h3 className="text-xl font-semibold text-gray-900">
            {therapy.name}
          </h3>
          <p className="mt-2 text-gray-600">{therapy.description}</p>
        </div>
        <div
          className="w-8 h-8 rounded-full aspect-square"
          style={{ backgroundColor: therapy.lightColor }}
        />
      </div>

      <div>
        <div className="mt-4 flex items-center gap-6 text-gray-600">
          <div className="flex items-center gap-2">
            <Thermometer className="w-5 h-5" />
            <span>
              {therapy.temperatureRange.min}°C - {therapy.temperatureRange.max}
              °C
            </span>
          </div>
          <div className="flex items-center gap-2">
            <Clock className="w-5 h-5" />
            <span>{therapy.duration} min</span>
          </div>
        </div>

        <a href={therapy.id}>
          <button
            onClick={onSelect}
            className={`mt-4 w-full py-2 px-4 rounded-lg font-medium transition-colors ${
              selected
                ? "bg-indigo-600 text-white hover:bg-indigo-700"
                : "bg-gray-100 text-gray-900 hover:bg-gray-200"
            }`}
          >
            {selected ? "Selected" : "Use This Therapy"}
          </button>
        </a>
      </div>
    </div>
  );
};

``````


### components/Meta.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/components/Meta.tsx
`relative_path`: components/Meta.tsx
`format`: Arbitrary Binary Data
`size`: 1494   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
export interface MetaProps {
  /** Title of the current page */
  title: string;
  /** Description of the current page */
  description: string;
  /** URL of the current page */
  href: string;
  /** URL of the cover image */
  imageUrl: string;
}

export default function Meta(props: MetaProps) {
  return (
    <>
      {/* HTML Meta Tags */}
      <title>{props.title}</title>
      <meta name="description" content={props.description} />

      {/* Google / Search Engine Tags */}
      <meta itemProp="name" content={props.title} />
      <meta itemProp="description" content={props.description} />
      {props.imageUrl && <meta itemProp="image" content={props.imageUrl} />}

      {/* Facebook Meta Tags */}
      <meta property="og:type" content="website" />
      <meta property="og:site_name" content={props.title} />
      <meta property="og:locale" content="en" />
      <meta property="og:title" content={props.title} />
      <meta property="og:description" content={props.description} />
      <meta property="og:url" content={props.href} />
      <meta property="og:image" content={props.imageUrl} />

      {/* Twitter Meta Tags */}
      <meta name="twitter:card" content="summary_large_image" />
      <meta name="twitter:title" content={props.title} />
      <meta name="twitter:description" content={props.description} />
      <meta name="twitter:image" content={props.imageUrl} />
    </>
  );
}

``````


### components/GitHubAvatarImg.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/components/GitHubAvatarImg.tsx
`relative_path`: components/GitHubAvatarImg.tsx
`format`: Arbitrary Binary Data
`size`: 835   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
export interface GitHubAvatarImgProps {
  /** The GitHub user's username */
  login: string;
  /** The height and width (1:1 ratio) of the image, in pixels */
  size: number;
  /** Additional classes */
  class?: string;
}

export default function GitHubAvatarImg(props: GitHubAvatarImgProps) {
  return (
    <img
      height={props.size}
      width={props.size}
      // Intrinsic size is 2x rendered size for Retina displays
      src={`https://avatars.githubusercontent.com/${props.login}?s=${
        props.size * 2
      }`}
      alt={`GitHub avatar of ${props.login}`}
      class={`rounded-full inline-block aspect-square size-[${props.size}px] ${
        props.class ?? ""
      }`}
      crossOrigin="anonymous"
      loading="lazy"
    />
  );
}

``````


### components/Footer.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/components/Footer.tsx
`relative_path`: components/Footer.tsx
`format`: Arbitrary Binary Data
`size`: 2884   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { SITE_NAME } from "@/utils/constants.ts";
import IconBrandDiscord from "tabler_icons_tsx/brand-discord.tsx";
import IconBrandGithub from "tabler_icons_tsx/brand-github.tsx";
import IconRss from "tabler_icons_tsx/rss.tsx";

function MadeWithFreshBadge() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="197" height="37" fill="none">
      <title>Made with Fresh</title>
      <rect width="196" height="36" x="0.5" y="0.5" fill="#ffffff" rx="5.5">
      </rect>
      <path
        fill="#be185d"
        d="M40.16 10.75c3.144 7.674 0 11.8-2.66 14.075.65 2.275-1.94 2.744-2.925 1.625-2.897.999-8.783.967-13-3.25-2.275-2.275-.5-7.336 5.525-10.725 5.2-2.925 10.4-4.55 13.06-1.726z"
      >
      </path>
      <path
        fill="#ffffff"
        stroke="#be185d"
        strokeWidth="0.65"
        d="M27.1 12.475c4.45-2.923 9.766-4.147 11.939-2.255 3.336 2.905-7.064 9.478-10.964 11.03-4.225 1.682-1.95 5.525-4.225 5.525-1.95 0-1.625-2.6-3.369-5.037-1.03-1.44 1.523-5.916 6.619-9.263z"
      >
      </path>
      <rect
        width="196"
        height="36"
        x="0.5"
        y="0.5"
        stroke="#D2D2D2"
        rx="5.5"
      >
      </rect>
    </svg>
  );
}

export default function Footer() {
  return (
    <footer class="bg-white text-gray-900 py-10 site-bar-styles flex-col md:flex-row mt-8">
      <p class="text-gray-800">© {SITE_NAME}</p>
      <nav class="nav-styles">
        <a
          href="/blog"
          class="text-primary hover:text-primary-dark link-styles data-[current]:!text-gray-900 data-[current]:dark:!text-gray-900"
        >
          Blog
        </a>
        <a
          href="/feed"
          aria-label="Deno Hunt RSS Feed"
          class="link-styles text-gray-800"
        >
          <IconRss class="size-6" />
        </a>
        <a
          href="https://discord.gg/deno"
          target="_blank"
          aria-label="Deno SaaSKit on Discord"
          class="link-styles text-gray-800"
        >
          <IconBrandDiscord class="size-6" />
        </a>
        <a
          href="https://github.com/denoland/saaskit"
          target="_blank"
          aria-label="Deno SaaSKit repo on GitHub"
          class="link-styles text-gray-800"
        >
          <IconBrandGithub class="size-6" />
        </a>
        <a href="https://fresh.deno.dev">
          <MadeWithFreshBadge />
        </a>
      </nav>
      <style>
        {`
          .text-primary {
            color: #be185d; 
          }
          .text-primary-dark {
            color: #9b1746;
          }
          .bg-white {
            background-color: #ffffff;
          }
          .text-gray-900 {
            color: #1f2937; 
          }
          .text-gray-800 {
            color: #374151;
          }
        `}
      </style>
    </footer>
  );
}

``````


### components/TemperatureRange.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/components/TemperatureRange.tsx
`relative_path`: components/TemperatureRange.tsx
`format`: Arbitrary Binary Data
`size`: 1222   


``````
interface TemperatureRangeProps {
  min: number;
  max: number;
  onChange: (min: number, max: number) => void;
}

export const TemperatureRange = ({
  min,
  max,
  onChange,
}: TemperatureRangeProps) => {
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-4">
        <label className="w-24">Min (°C):</label>
        <input
          type="range"
          min="0"
          max="30"
          value={min}
          onChange={(e) =>
            onChange(Number((e.target as HTMLInputElement).value), max)
          }
          className="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
        />
        <span className="w-12 text-right">{min}°C</span>
      </div>

      <div className="flex items-center gap-4">
        <label className="w-24">Max (°C):</label>
        <input
          type="range"
          min="0"
          max="30"
          value={max}
          onChange={(e) =>
            onChange(min, Number((e.target as HTMLInputElement).value))
          }
          className="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
        />
        <span className="w-12 text-right">{max}°C</span>
      </div>
    </div>
  );
};

``````


### components/Head.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/components/Head.tsx
`relative_path`: components/Head.tsx
`format`: Arbitrary Binary Data
`size`: 755   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { Head as _Head } from "$fresh/runtime.ts";
import Meta, { type MetaProps } from "./Meta.tsx";
import { SITE_DESCRIPTION, SITE_NAME } from "@/utils/constants.ts";
import { ComponentChildren } from "preact";

export type HeadProps =
  & Partial<Omit<MetaProps, "href">>
  & Pick<MetaProps, "href">
  & {
    children?: ComponentChildren;
  };

export default function Head(props: HeadProps) {
  return (
    <_Head>
      <Meta
        title={props?.title ? `${props.title} ▲ ${SITE_NAME}` : SITE_NAME}
        description={props?.description ?? SITE_DESCRIPTION}
        href={props.href}
        imageUrl="/cover.png"
      />
      {props.children}
    </_Head>
  );
}

``````


### components/TabsBar.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/components/TabsBar.tsx
`relative_path`: components/TabsBar.tsx
`format`: Arbitrary Binary Data
`size`: 1066   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { ComponentChildren } from "preact";

export interface TabItemProps {
  /** Path of the item's URL */
  path: string;
  /** Whether the user is on the item's URL */
  active: boolean;
  children?: ComponentChildren;
}

export function TabItem(props: TabItemProps) {
  return (
    <a
      href={props.path}
      class={`px-4 py-2 rounded-lg ${
        props.active
          ? "bg-primary text-white dark:bg-secondary dark:text-base-100"
          : "text-neutral hover:bg-primary hover:text-base-100"
      } link-styles`}
    >
      {props.children}
    </a>
  );
}

export interface TabsBarProps {
  links: {
    path: string;
    innerText: string;
  }[];
  currentPath: string;
}

export default function TabsBar(props: TabsBarProps) {
  return (
    <div class="flex flex-row w-full mb-8">
      {props.links.map((link) => (
        <TabItem path={link.path} active={link.path === props.currentPath}>
          {link.innerText}
        </TabItem>
      ))}
    </div>
  );
}

``````


### components/Header.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/components/Header.tsx
`relative_path`: components/Header.tsx
`format`: Arbitrary Binary Data
`size`: 4484   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
// import { SITE_NAME } from "@/utils/constants.ts";
// import { isStripeEnabled } from "@/utils/stripe.ts";
// import IconX from "tabler_icons_tsx/x.tsx";
// import IconMenu from "tabler_icons_tsx/menu-2.tsx";
// import { User } from "@/utils/db.ts";
import { CreditCard, LogIn, MapPin, Menu, User, X } from "lucide-preact"; // Correct import

export interface HeaderProps {
  /** Currently signed-in user */
  sessionUser?: User;
  /**
   * URL of the current page. This is used for highlighting the currently
   * active page in navigation.
   */
  url: URL;
}

export default function Header(props: HeaderProps) {
  // Extract the `isMenuOpen` query parameter from the URL
  const isMenuOpen = props.url.searchParams.get("menu") === "open";

  return (
    <header className="bg-white shadow-sm">
      <nav className="mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          {/* Logo and brand */}
          <div className="flex items-center">
            <a href="/" className="flex items-center">
              <span className="text-2xl font-bold text-indigo-600">Vacuul</span>
            </a>
          </div>

          {/* Desktop navigation */}
          <div className="hidden md:flex md:items-center md:space-x-8">
            <a
              href="/map"
              className="flex items-center text-gray-600 hover:text-gray-900"
            >
              <MapPin className="w-5 h-5 mr-1" />
              Find Location
            </a>
            <a
              href="/pricing"
              className="flex items-center text-gray-600 hover:text-gray-900"
            >
              <CreditCard className="w-5 h-5 mr-1" />
              Pricing
            </a>
            {props.sessionUser
              ? (
                <a
                  href="/user-profile"
                  className="flex items-center text-gray-600 hover:text-gray-900"
                >
                  <User className="w-5 h-5 mr-1" />
                  Account
                </a>
              )
              : (
                <a
                  href="/login"
                  className="flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white hover:bg-indigo-700"
                >
                  <LogIn className="w-5 h-5 mr-1" />
                  Sign In
                </a>
              )}
          </div>

          {/* Mobile menu button */}
          <div className="flex items-center md:hidden">
            <a
              href={isMenuOpen
                ? `${props.url.pathname}`
                : `${props.url.pathname}?menu=open`}
              className="inline-flex items-center justify-center p-2 rounded-md text-gray-600 hover:text-gray-900 hover:bg-gray-100"
            >
              {isMenuOpen
                ? <X className="w-6 h-6" />
                : <Menu className="w-6 h-6" />}
            </a>
          </div>
        </div>

        {isMenuOpen && (
          <div className="md:hidden py-4">
            <div className="flex flex-col space-y-4">
              <a
                href="/map"
                className="flex items-center text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md hover:bg-gray-100"
              >
                <MapPin className="w-5 h-5 mr-2" />
                Find Location
              </a>
              <a
                href="/pricing"
                className="flex items-center text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md hover:bg-gray-100"
              >
                <CreditCard className="w-5 h-5 mr-2" />
                Pricing
              </a>
              {props.sessionUser
                ? (
                  <a
                    href="/account"
                    className="flex items-center text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md hover:bg-gray-100"
                  >
                    <User className="w-5 h-5 mr-2" />
                    Account
                  </a>
                )
                : (
                  <a
                    href="/login"
                    className="flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white hover:bg-indigo-700"
                  >
                    <LogIn className="w-5 h-5 mr-2" />
                    Sign In
                  </a>
                )}
            </div>
          </div>
        )}
      </nav>
    </header>
  );
}

``````


### components/PremiumBadge.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/components/PremiumBadge.tsx
`relative_path`: components/PremiumBadge.tsx
`format`: Arbitrary Binary Data
`size`: 3173   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
interface PremiumBadgeProps {
  class?: string;
}

export function PremiumBadge(props: PremiumBadgeProps) {
  return (
    <svg
      width="30"
      height="30"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      viewBox={"0 0 30 30"}
      {...props}
    >
      <title>
        Deno Hunt premium user
      </title>
      <g clip-path="url(#prefix__clip0_802_86)">
        <path
          d="M15 0c8.284 0 15 6.716 15 15 0 8.284-6.716 15-15 15-8.284 0-15-6.716-15-15C0 6.716 6.716 0 15 0z"
          fill="url(#prefix__paint0_linear_802_86)"
        />
        <mask
          id="prefix__a"
          style="mask-type:alpha"
          maskUnits="userSpaceOnUse"
          x="0"
          y="0"
          width="30"
          height="30"
        >
          <path
            d="M15 .5C23.008.5 29.5 6.992 29.5 15S23.008 29.5 15 29.5.5 23.008.5 15 6.992.5 15 .5z"
            fill="#9E5656"
            stroke="#FF2B2B"
          />
        </mask>
        <g mask="url(#prefix__a)">
          <path
            d="M16.598 31.144l.142.978.98-.13c3.86-.517 7.42-2.443 10.024-5.34l.361-.402-.138-.523-2.614-9.86h0l-.003-.007c-.655-2.395-1.482-4.878-3.772-6.69h0C19.756 7.73 17.47 7 14.876 7 9.54 7 5 10.438 5 15.132c0 2.233 1.088 4.063 2.928 5.278 1.759 1.162 4.173 1.75 6.977 1.723a8.847 8.847 0 01.038.112l.029.11c.023.094.05.215.082.36.063.29.137.66.219 1.078.11.567.23 1.207.347 1.833.056.298.11.592.164.874l.042.22c.28 1.487.558 2.954.772 4.424z"
            fill="url(#prefix__paint1_linear_802_86)"
            stroke="#DF7F26"
            stroke-width="2"
          />
          <path
            d="M15.124 12a1.124 1.124 0 110 2.248 1.124 1.124 0 010-2.248z"
            fill="#DF7F26"
          />
        </g>
        <path
          d="M15 1c7.732 0 14 6.268 14 14s-6.268 14-14 14S1 22.732 1 15 7.268 1 15 1z"
          stroke="#AB5F1A"
          stroke-width="2"
        />
        <path
          d="M3.5 12C5 6.5 10 3.5 15 3M7.5 15.5c0-4 4.5-6.5 7-6.5M17 22l.5 3"
          stroke="#fff"
          stroke-width="2"
          stroke-linecap="round"
        />
      </g>
      <defs>
        <linearGradient
          id="prefix__paint0_linear_802_86"
          x1="6"
          y1="2.5"
          x2="23.5"
          y2="36.5"
          gradientUnits="userSpaceOnUse"
        >
          <stop stop-color="#FFED8F" />
          <stop offset=".276" stop-color="#FFDF70" />
          <stop offset=".536" stop-color="#F8BB1E" />
          <stop offset=".781" stop-color="#FBEB5C" />
          <stop offset="1" stop-color="#FFEE93" />
        </linearGradient>
        <linearGradient
          id="prefix__paint1_linear_802_86"
          x1="14"
          y1="10.5"
          x2="28"
          y2="31"
          gradientUnits="userSpaceOnUse"
        >
          <stop stop-color="#FFF3B7" />
          <stop offset=".422" stop-color="#FFCA63" />
          <stop offset="1" stop-color="#FAD258" />
        </linearGradient>
        <clipPath id="prefix__clip0_802_86">
          <path fill="#fff" d="M0 0h30v30H0z" />
        </clipPath>
      </defs>
    </svg>
  );
}

``````


### components/ColorPicker.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/components/ColorPicker.tsx
`relative_path`: components/ColorPicker.tsx
`format`: Arbitrary Binary Data
`size`: 950   


``````
interface ColorPickerProps {
  value: string;
  onChange: (color: string) => void;
}

const predefinedColors = [
  { value: "#3B82F6", label: "Blue" },
  { value: "#EF4444", label: "Red" },
  { value: "#10B981", label: "Green" },
  { value: "#8B5CF6", label: "Purple" },
  { value: "#F59E0B", label: "Yellow" },
];

export const ColorPicker = ({ value, onChange }: ColorPickerProps) => {
  return (
    <div className="flex flex-wrap gap-3">
      {predefinedColors.map((color) => (
        <button
          key={color.value}
          onClick={() => onChange(color.value)}
          className={`w-12 h-12 rounded-full transition-transform ${
            value === color.value
              ? "scale-110 ring-2 ring-offset-2 ring-indigo-600"
              : ""
          }`}
          style={{ backgroundColor: color.value }}
          title={color.label}
          aria-label={`Select ${color.label} color`}
        />
      ))}
    </div>
  );
};

``````


### static/favicon.ico
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/static/favicon.ico
`relative_path`: static/favicon.ico
`format`: Windows Icon
`size`: 9662   




### static/styles.css
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/static/styles.css
`relative_path`: static/styles.css
`format`: Arbitrary Binary Data
`size`: 1074   


``````
@tailwind base;
@tailwind components;
@tailwind utilities;

.button-styles {
  @apply px-4 py-2 bg-primary text-white rounded-lg border border-primary
    transition duration-100 disabled:opacity-50 disabled:cursor-not-allowed
    hover:bg-transparent hover:text-primary;
}

.input-styles {
  @apply px-4 py-2 bg-transparent rounded-lg outline-none border border-neutral
    hover:border-secondary transition duration-100 disabled:opacity-50
    disabled:cursor-not-allowed hover:border-white;
}

.site-bar-styles {
  @apply flex justify-between p-4 gap-4;
}

.nav-styles {
  @apply flex flex-wrap justify-start gap-x-8 gap-y-4 items-center
    justify-between h-full;
}

.nav-item {
  @apply text-neutral px-3 py-4 sm:py-2;
}

.link-styles {
  @apply text-gray-500 transition duration-100 hover:text-black
    hover:dark:text-white;
}

.heading-styles {
  @apply text-3xl font-bold text-primary;
}

.heading-with-margin-styles {
  @apply text-3xl font-bold mb-8 text-primary;
}

body {
  @apply bg-base-100 text-neutral;
}

body.dark {
  @apply bg-base-100 text-neutral;
}

``````


### static/vacuul_logo.webp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/static/vacuul_logo.webp
`relative_path`: static/vacuul_logo.webp
`format`: WebP
`size`: 13828   




### static/logo.webp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/static/logo.webp
`relative_path`: static/logo.webp
`format`: WebP
`size`: 3498   




### static/cover.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/static/cover.png
`relative_path`: static/cover.png
`format`: Portable Network Graphics
`size`: 36359   




### CONTRIBUTING.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/CONTRIBUTING.md
`relative_path`: CONTRIBUTING.md
`format`: Arbitrary Binary Data
`size`: 848   


``````
# Contributing

## Ways to Contribute

- Report bugs
- Implement performance improvements
- Refine documentation
- Suggest new features and design improvements
- Link to your SaaSKit-related project

## Creating an Issue

- Make sure that an issue doesn't already exist (avoid duplicates)
- Feel free to ask questions in the
  [#saaskit Discord channel](https://discord.com/channels/684898665143206084/712010403302866974)

## Submitting a Pull Request

- Follow the
  [Deno Style Guide](https://deno.land/manual/references/contributing/style_guide)
- A pull request must address a given issue. If an issue does not exist, create
  one
- Tests must be provided for new functionality

## Design Rules

Follow
[CocoaPods design rules](https://github.com/CocoaPods/CocoaPods/wiki/Communication-&-Design-Rules#design-rules)
which apply to this project.

``````


### docker-compose.yml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/docker-compose.yml
`relative_path`: docker-compose.yml
`format`: Arbitrary Binary Data
`size`: 465   


``````
version: "3"

services:
  web:
    build: .
    container_name: deno-saaskit
    image: deno-image
    environment:
      - DENO_DEPLOYMENT_ID=${DENO_DEPLOYMENT_ID}
      - GITHUB_CLIENT_ID=${GITHUB_CLIENT_ID}
      - GITHUB_CLIENT_SECRET=${GITHUB_CLIENT_SECRET}
      - STRIPE_SECRET_KEY=${STRIPE_SECRET_KEY}
      - STRIPE_WEBHOOK_SECRET=${STRIPE_WEBHOOK_SECRET}
      - STRIPE_PREMIUM_PLAN_PRICE_ID=${STRIPE_PREMIUM_PLAN_PRICE_ID}
    ports:
      - "8000:8000"

``````


### data/therapies.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/data/therapies.ts
`relative_path`: data/therapies.ts
`format`: Arbitrary Binary Data
`size`: 821   


``````
import { TherapyOption } from "@/types/therapy.ts";

export const therapyOptions: TherapyOption[] = [
  {
    id: "relaxation",
    name: "Relaxation",
    lightColor: "#3B82F6", // blue-500
    temperatureRange: { min: 10, max: 20 },
    description:
      "Gentle therapy designed to promote relaxation and stress relief",
    duration: 15,
  },
  {
    id: "strength",
    name: "Strength",
    lightColor: "#EF4444", // red-500
    temperatureRange: { min: 5, max: 15 },
    description:
      "Intensive therapy focused on muscle recovery and strength building",
    duration: 30,
  },
  {
    id: "energy",
    name: "Energy",
    lightColor: "#10B981", // emerald-500
    temperatureRange: { min: 15, max: 25 },
    description: "Energizing therapy to boost vitality and mental clarity",
    duration: 20,
  },
];

``````


### routes/index.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/index.tsx
`relative_path`: routes/index.tsx
`format`: Arbitrary Binary Data
`size`: 5960   




### routes/pricing.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/pricing.tsx
`relative_path`: routes/pricing.tsx
`format`: Arbitrary Binary Data
`size`: 5484   




### routes/end-of-therapy.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/end-of-therapy.tsx
`relative_path`: routes/end-of-therapy.tsx
`format`: Arbitrary Binary Data
`size`: 184   


``````
import { defineRoute } from "$fresh/server.ts";

export default defineRoute((_req, ctx) => {
  return (
    <div>
      <h1>End of Therapy and Feedback Screen</h1>
    </div>
  );
});

``````


### routes/login.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/login.tsx
`relative_path`: routes/login.tsx
`format`: Arbitrary Binary Data
`size`: 3491   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { defineRoute } from "$fresh/server.ts";
import { redirect } from "kv_oauth/lib/_http.ts";
import type { State } from "@/plugins/session.ts";

export default defineRoute<State>((_req, ctx) => {
    const isSignedIn = ctx.state.sessionUser !== undefined;

    if (isSignedIn) {
        return redirect("/dashboard");
    }

    return (
        <>
            <section class="min-h-screen flex items-center justify-center bg-indigo-100">
                <div class="max-w-md w-full bg-white p-8 rounded-lg shadow-lg">
                    <h2 class="text-2xl font-bold mb-6 text-center text-gray-900">
                        Sign in to your account
                    </h2>
                    <p class="text-center mb-4 text-gray-600">
                        We currently support login via Google and GitHub only.
                        More login methods will be added soon.
                    </p>

                    <div class="space-y-4">
                        <a
                            href="/auth/google"
                            class="flex justify-center items-center w-full text-white bg-indigo-600 hover:bg-indigo-700 font-medium rounded-lg text-sm px-5 py-2.5 text-center transition"
                        >
                            <svg
                                class="w-6 h-6 mr-3 text-white"
                                xmlns="http://www.w3.org/2000/svg"
                                fill="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path d="M12 22a10 10 0 1 1 0-20 10 10 0 0 1 0 20Zm0-3.4c4 0 7.4-3.4 7.4-7.4s-3.4-7.4-7.4-7.4S4.6 7.6 4.6 11.6c0 4 3.4 7.4 7.4 7.4Z" />
                            </svg>
                            Login with Google
                        </a>

                        <a
                            href="/auth/github"
                            class="flex justify-center items-center w-full text-white bg-gray-800 hover:bg-gray-900 font-medium rounded-lg text-sm px-5 py-2.5 text-center transition"
                        >
                            <svg
                                class="w-6 h-6 mr-3 text-white"
                                xmlns="http://www.w3.org/2000/svg"
                                fill="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path d="M12 2c-2.4 0-4.7.9-6.5 2.4a10.5 10.5 0 0 0-2 13.1A10 10 0 0 0 8.7 22c.5 0 .7-.2.7-.5v-2c-2.8.7-3.4-1.1-3.4-1.1-.1-.6-.5-1.2-1-1.5-1-.7 0-.7 0-.7a2 2 0 0 1 1.5 1.1 2.2 2.2 0 0 0 1.3 1 2 2 0 0 0 1.6-.1c0-.6.3-1 .7-1.4-2.2-.3-4.6-1.2-4.6-5 0-1.1.4-2 1-2.8a4 4 0 0 1 .2-2.7s.8-.3 2.7 1c1.6-.5 3.4-.5 5 0 2-1.3 2.8-1 2.8-1 .3.8.4 1.8 0 2.7a4 4 0 0 1 1 2.7c0 4-2.3 4.8-4.5 5a2.5 2.5 0 0 1 .7 2v2.8c0 .3.2.6.7.5a10 10 0 0 0 5.4-4.4 10.5 10.5 0 0 0-2.1-13.2A9.8 9.8 0 0 0 12 2Z" />
                            </svg>
                            Login with GitHub
                        </a>
                    </div>

                    <p class="mt-4 text-center text-sm text-gray-500">
                        Don't have an account?{" "}
                        <a href="#" class="text-indigo-600 hover:underline">
                            Register
                        </a>
                    </p>
                </div>
            </section>
        </>
    );
});

``````


### routes/therapy-progress.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/therapy-progress.tsx
`relative_path`: routes/therapy-progress.tsx
`format`: Arbitrary Binary Data
`size`: 193   


``````
import { defineRoute } from "$fresh/server.ts";
import { TherapyProgress } from "@/islands/TherapyProgress.tsx";

export default defineRoute((_req, _ctx) => {
  return <TherapyProgress />;
});

``````


### routes/booking/payment.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/booking/payment.tsx
`relative_path`: routes/booking/payment.tsx
`format`: Empty
`size`: 0   




### routes/booking/success.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/booking/success.tsx
`relative_path`: routes/booking/success.tsx
`format`: Empty
`size`: 0   




### routes/dashboard/index.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/dashboard/index.tsx
`relative_path`: routes/dashboard/index.tsx
`format`: Arbitrary Binary Data
`size`: 259   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { redirect } from "@/utils/http.ts";
import { Handlers } from "$fresh/server.ts";

export const handler: Handlers = {
  GET() {
    return redirect("/dashboard/stats");
  },
};

``````


### routes/dashboard/stats.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/dashboard/stats.tsx
`relative_path`: routes/dashboard/stats.tsx
`format`: Arbitrary Binary Data
`size`: 2487   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import Chart from "@/islands/Chart.tsx";
import Head from "@/components/Head.tsx";
import TabsBar from "@/components/TabsBar.tsx";
import { defineRoute } from "$fresh/server.ts";
import { Partial } from "$fresh/runtime.ts";

function randomNumbers(length: number) {
  return Array.from({ length }, () => Math.floor(Math.random() * 1000));
}

export default defineRoute((_req, ctx) => {
  const labels = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
  ];
  const datasets = [
    {
      label: "Site visits",
      data: randomNumbers(labels.length),
      borderColor: "#be185d",
    },
    {
      label: "Users created",
      data: randomNumbers(labels.length),
      borderColor: "#e85d04",
    },
    {
      label: "Items created",
      data: randomNumbers(labels.length),
      borderColor: "#219ebc",
    },
    {
      label: "Votes",
      data: randomNumbers(labels.length),
      borderColor: "#4338ca",
    },
  ];

  return (
    <>
      <Head title="Dashboard" href={ctx.url.href} />
      <main class="flex-1 p-4 flex flex-col f-client-nav">
        <h1 class="heading-with-margin-styles">Dashboard</h1>
        <TabsBar
          links={[{
            path: "/dashboard/stats",
            innerText: "Stats",
          }, {
            path: "/dashboard/users",
            innerText: "Users",
          }]}
          currentPath={ctx.url.pathname}
        />
        <Partial name="stats">
          <div class="flex-1 relative">
            <Chart
              type="line"
              options={{
                maintainAspectRatio: false,
                interaction: {
                  intersect: false,
                  mode: "index",
                },
                scales: {
                  x: {
                    grid: { display: false },
                  },
                  y: {
                    beginAtZero: true,
                    grid: { display: false },
                    ticks: { precision: 0 },
                  },
                },
              }}
              data={{
                labels,
                datasets: datasets.map((dataset) => ({
                  ...dataset,
                  pointRadius: 0,
                  cubicInterpolationMode: "monotone",
                })),
              }}
            />
          </div>
        </Partial>
      </main>
    </>
  );
});

``````


### routes/dashboard/users.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/dashboard/users.tsx
`relative_path`: routes/dashboard/users.tsx
`format`: Arbitrary Binary Data
`size`: 1088   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import Head from "@/components/Head.tsx";
import TabsBar from "@/components/TabsBar.tsx";
import UsersTable from "@/islands/UsersTable.tsx";
import { defineRoute } from "$fresh/server.ts";
import { Partial } from "$fresh/runtime.ts";

export default defineRoute((_req, ctx) => {
  const endpoint = "/api/users";

  return (
    <>
      <Head title="Users" href={ctx.url.href}>
        <link
          as="fetch"
          crossOrigin="anonymous"
          href={endpoint}
          rel="preload"
        />
      </Head>
      <main class="flex-1 p-4 f-client-nav">
        <h1 class="heading-with-margin-styles">Dashboard</h1>
        <TabsBar
          links={[{
            path: "/dashboard/stats",
            innerText: "Stats",
          }, {
            path: "/dashboard/users",
            innerText: "Users",
          }]}
          currentPath={ctx.url.pathname}
        />
        <Partial name="users">
          <UsersTable endpoint={endpoint} />
        </Partial>
      </main>
    </>
  );
});

``````


### routes/submit.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/submit.tsx
`relative_path`: routes/submit.tsx
`format`: Arbitrary Binary Data
`size`: 4447   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import Head from "@/components/Head.tsx";
import IconCheckCircle from "tabler_icons_tsx/circle-check.tsx";
import IconCircleX from "tabler_icons_tsx/circle-x.tsx";
import { defineRoute, Handlers } from "$fresh/server.ts";
import { createItem } from "@/utils/db.ts";
import { redirect } from "@/utils/http.ts";
import {
  assertSignedIn,
  type SignedInState,
  State,
} from "@/plugins/session.ts";
import { ulid } from "$std/ulid/mod.ts";
import IconInfo from "tabler_icons_tsx/info-circle.tsx";

const SUBMIT_STYLES =
  "w-full text-white text-center rounded-[7px] transition duration-300 px-4 py-2 block hover:bg-white hover:text-black hover:dark:bg-gray-900 hover:dark:!text-white";

export const handler: Handlers<undefined, SignedInState> = {
  async POST(req, ctx) {
    assertSignedIn(ctx);

    const form = await req.formData();
    const title = form.get("title");
    const url = form.get("url");

    if (
      typeof url !== "string" ||
      !URL.canParse(url) ||
      typeof title !== "string" ||
      title === ""
    ) {
      return redirect("/submit?error");
    }

    await createItem({
      id: ulid(),
      userLogin: ctx.state.sessionUser.login,
      title,
      url,
      score: 0,
    });
    return redirect("/");
  },
};

export default defineRoute<State>((_req, ctx) => {
  return (
    <>
      <Head title="Submit" href={ctx.url.href} />
      <main class="flex-1 flex flex-col justify-center mx-auto w-full space-y-16 p-4 max-w-6xl">
        <div class="text-center">
          <h1 class="heading-styles">Share your project</h1>
          <p class="text-gray-500">
            Let the community know about your Deno-related blog post, video or
            module!
          </p>
        </div>
        <div class="flex flex-col md:flex-row gap-8 md:gap-16 md:items-center">
          <div class="flex-1 space-y-6">
            <p>
              <IconCircleX class="inline-block mr-2" />
              <strong>Don't</strong> post duplicate content
            </p>
            <p>
              <IconCircleX class="inline-block mr-2" />
              <strong>Don't</strong> share dummy or test posts
            </p>
            <div>
              <IconCheckCircle class="inline-block mr-2" />
              <strong>Do</strong> include a description with your title.
              <div class="text-sm text-gray-500">
                E.g. “Deno Hunt: the best place to share your Deno project”
              </div>
            </div>
          </div>
          <form class="flex-1 flex flex-col justify-center" method="post">
            <div>
              <label
                htmlFor="submit_title"
                class="block text-sm font-medium leading-6 text-gray-900"
              >
                Title
              </label>
              <input
                id="submit_title"
                class="input-styles w-full mt-2"
                type="text"
                name="title"
                required
                placeholder="Deno Hunt: the best place to share your Deno project"
                disabled={!ctx.state.sessionUser}
              />
            </div>

            <div class="mt-4">
              <label
                htmlFor="submit_url"
                class="block text-sm font-medium leading-6 text-gray-900"
              >
                URL
              </label>
              <input
                id="submit_url"
                class="input-styles w-full mt-2"
                type="url"
                name="url"
                required
                placeholder="https://my-awesome-project.com"
                disabled={!ctx.state.sessionUser}
              />
            </div>
            {ctx.url.searchParams.has("error") && (
              <div class="w-full text-red-500 mt-4">
                <IconInfo class="inline-block" /> Title and valid URL are
                required
              </div>
            )}
            <div class="w-full rounded-lg bg-gradient-to-tr from-secondary to-primary p-px mt-8">
              {!ctx.state.sessionUser ? (
                <a href="/login" class={SUBMIT_STYLES}>
                  Sign in to submit &#8250;
                </a>
              ) : (
                <button class={SUBMIT_STYLES}>Submit</button>
              )}
            </div>
          </form>
        </div>
      </main>
    </>
  );
});

``````


### routes/users/[login].tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/users/[login].tsx
`relative_path`: routes/users/[login].tsx
`format`: Arbitrary Binary Data
`size`: 2558   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type { State } from "@/plugins/session.ts";
import { getUser } from "@/utils/db.ts";
import IconBrandGithub from "tabler_icons_tsx/brand-github.tsx";
import Head from "@/components/Head.tsx";
import ItemsList from "@/islands/ItemsList.tsx";
import { defineRoute } from "$fresh/server.ts";
import { PremiumBadge } from "@/components/PremiumBadge.tsx";

interface UserProfileProps {
  login: string;
  isSubscribed: boolean;
  picture?: string;
  name?: string;
}

function UserProfile(props: UserProfileProps) {
  return (
    <div class="flex flex-col items-center w-[16rem]">
      {props.picture
        ? (
          <img
            src={props.picture}
            alt="Profile Picture"
            class="rounded-full w-48 h-48"
          />
        )
        : (
          <img
            src={`https://github.com/${props.login}.png`}
            alt="GitHub Profile Picture"
            class="rounded-full w-48 h-48"
          />
        )}
      <div class="flex gap-x-2 px-4 mt-4 items-center">
        <div class="font-semibold text-xl">{props.name || props.login}</div>
        {props.isSubscribed && <PremiumBadge class="size-6 inline" />}
        <a
          href={`https://github.com/${props.login}`}
          aria-label={`${props.login}'s GitHub profile`}
          class="link-styles"
          target="_blank"
        >
          <IconBrandGithub class="w-6" />
        </a>
      </div>
    </div>
  );
}

export default defineRoute<State>(
  async (_req, ctx) => {
    const { login } = ctx.params;
    const user = await getUser(login);
    if (user === null) return await ctx.renderNotFound();

    const isSignedIn = ctx.state.sessionUser !== undefined;
    const endpoint = `/api/users/${login}/items`;

    return (
      <>
        <Head title={user.login} href={ctx.url.href}>
          <link
            as="fetch"
            crossOrigin="anonymous"
            href={endpoint}
            rel="preload"
          />
          {isSignedIn && (
            <link
              as="fetch"
              crossOrigin="anonymous"
              href="/api/me/votes"
              rel="preload"
            />
          )}
        </Head>
        <main class="flex-1 p-4 flex flex-col md:flex-row gap-8">
          <div class="flex justify-center p-4">
            <UserProfile {...user} />
          </div>
          <ItemsList
            endpoint={endpoint}
            isSignedIn={isSignedIn}
          />
        </main>
      </>
    );
  },
);

``````


### routes/api/stripe-webhooks.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/api/stripe-webhooks.ts
`relative_path`: routes/api/stripe-webhooks.ts
`format`: Arbitrary Binary Data
`size`: 2623   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { type Handlers } from "$fresh/server.ts";
import { STATUS_CODE } from "$std/http/status.ts";
import { isStripeEnabled, stripe } from "@/utils/stripe.ts";
import Stripe from "stripe";
import { getUserByStripeCustomer, updateUser } from "@/utils/db.ts";
import { BadRequestError } from "@/utils/http.ts";

const cryptoProvider = Stripe.createSubtleCryptoProvider();
export const handler: Handlers = {
  /**
   * Handles Stripe webhooks requests when a user subscribes
   * (`customer.subscription.created`) or cancels
   * (`customer.subscription.deleted`) the "Premium Plan".
   *
   * @see {@link https://github.com/stripe-samples/stripe-node-deno-samples/blob/2d571b20cd88f1c1f02185483729a37210484c68/webhook-signing/main.js}
   */
  async POST(req) {
    if (!isStripeEnabled()) throw new Deno.errors.NotFound("Not Found");

    /** @see {@link https://stripe.com/docs/webhooks#verify-events} */
    const body = await req.text();
    const signature = req.headers.get("stripe-signature");
    if (signature === null) {
      throw new BadRequestError("`Stripe-Signature` header is missing");
    }
    const signingSecret = Deno.env.get("STRIPE_WEBHOOK_SECRET");
    if (signingSecret === undefined) {
      throw new Error(
        "`STRIPE_WEBHOOK_SECRET` environment variable is not set",
      );
    }

    let event: Stripe.Event;
    try {
      event = await stripe.webhooks.constructEventAsync(
        body,
        signature,
        signingSecret,
        undefined,
        cryptoProvider,
      );
    } catch (error) {
      throw new BadRequestError((error as Error).message);
    }

    // @ts-ignore: Property 'customer' actually does exist on type 'Object'
    const { customer } = event.data.object;

    switch (event.type) {
      case "customer.subscription.created": {
        const user = await getUserByStripeCustomer(customer);
        if (user === null) {
          throw new Deno.errors.NotFound("User not found");
        }

        await updateUser({ ...user, isSubscribed: true });
        return new Response(null, { status: STATUS_CODE.Created });
      }
      case "customer.subscription.deleted": {
        const user = await getUserByStripeCustomer(customer);
        if (user === null) {
          throw new Deno.errors.NotFound("User not found");
        }

        await updateUser({ ...user, isSubscribed: false });
        return new Response(null, { status: STATUS_CODE.Accepted });
      }
      default: {
        throw new BadRequestError("Event type not supported");
      }
    }
  },
};

``````


### routes/api/vote.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/api/vote.ts
`relative_path`: routes/api/vote.ts
`format`: Arbitrary Binary Data
`size`: 748   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { type Handlers } from "$fresh/server.ts";
import { STATUS_CODE } from "$std/http/status.ts";
import type { SignedInState } from "@/plugins/session.ts";
import { createVote } from "@/utils/db.ts";
import { BadRequestError } from "@/utils/http.ts";

export const handler: Handlers<undefined, SignedInState> = {
  async POST(req, ctx) {
    const itemId = new URL(req.url).searchParams.get("item_id");
    if (itemId === null) {
      throw new BadRequestError("`item_id` URL parameter missing");
    }

    await createVote({
      itemId,
      userLogin: ctx.state.sessionUser.login,
    });

    return new Response(null, { status: STATUS_CODE.Created });
  },
};

``````


### routes/api/me/votes.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/api/me/votes.ts
`relative_path`: routes/api/me/votes.ts
`format`: Arbitrary Binary Data
`size`: 489   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type { Handlers } from "$fresh/server.ts";
import { collectValues, listItemsVotedByUser } from "@/utils/db.ts";
import { SignedInState } from "@/plugins/session.ts";

export const handler: Handlers<undefined, SignedInState> = {
  async GET(_req, ctx) {
    const iter = listItemsVotedByUser(ctx.state.sessionUser.login);
    const items = await collectValues(iter);
    return Response.json(items);
  },
};

``````


### routes/api/users/[login]/items.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/api/users/[login]/items.ts
`relative_path`: routes/api/users/[login]/items.ts
`format`: Arbitrary Binary Data
`size`: 682   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type { Handlers } from "$fresh/server.ts";
import { collectValues, getUser, listItemsByUser } from "@/utils/db.ts";
import { getCursor } from "@/utils/http.ts";

export const handler: Handlers = {
  async GET(req, ctx) {
    const user = await getUser(ctx.params.login);
    if (user === null) throw new Deno.errors.NotFound("User not found");

    const url = new URL(req.url);
    const iter = listItemsByUser(ctx.params.login, {
      cursor: getCursor(url),
      limit: 10,
    });
    const values = await collectValues(iter);
    return Response.json({ values, cursor: iter.cursor });
  },
};

``````


### routes/api/users/[login]/index.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/api/users/[login]/index.ts
`relative_path`: routes/api/users/[login]/index.ts
`format`: Arbitrary Binary Data
`size`: 390   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type { Handlers } from "$fresh/server.ts";
import { getUser } from "@/utils/db.ts";

export const handler: Handlers = {
  async GET(_req, ctx) {
    const user = await getUser(ctx.params.login);
    if (user === null) throw new Deno.errors.NotFound("User not found");
    return Response.json(user);
  },
};

``````


### routes/api/users/index.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/api/users/index.ts
`relative_path`: routes/api/users/index.ts
`format`: Arbitrary Binary Data
`size`: 514   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type { Handlers } from "$fresh/server.ts";
import { collectValues, listUsers } from "@/utils/db.ts";
import { getCursor } from "@/utils/http.ts";

export const handler: Handlers = {
  async GET(req) {
    const url = new URL(req.url);
    const iter = listUsers({
      cursor: getCursor(url),
      limit: 10,
    });
    const values = await collectValues(iter);
    return Response.json({ values, cursor: iter.cursor });
  },
};

``````


### routes/api/items/[id].ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/api/items/[id].ts
`relative_path`: routes/api/items/[id].ts
`format`: Arbitrary Binary Data
`size`: 387   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import type { Handlers } from "$fresh/server.ts";
import { getItem } from "@/utils/db.ts";

export const handler: Handlers = {
  async GET(_req, ctx) {
    const item = await getItem(ctx.params.id);
    if (item === null) throw new Deno.errors.NotFound("Item not found");
    return Response.json(item);
  },
};

``````


### routes/api/items/index.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/api/items/index.ts
`relative_path`: routes/api/items/index.ts
`format`: Arbitrary Binary Data
`size`: 536   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.

import { collectValues, listItems } from "@/utils/db.ts";
import { getCursor } from "@/utils/http.ts";
import type { Handlers } from "$fresh/server.ts";

export const handler: Handlers = {
  async GET(req) {
    const url = new URL(req.url);
    const iter = listItems({
      cursor: getCursor(url),
      limit: 10,
      reverse: true,
    });
    const values = await collectValues(iter);
    return Response.json({ values, cursor: iter.cursor });
  },
};

``````


### routes/_500.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/_500.tsx
`relative_path`: routes/_500.tsx
`format`: Arbitrary Binary Data
`size`: 492   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { PageProps } from "$fresh/server.ts";

export default function Error500Page(props: PageProps) {
  return (
    <main class="flex flex-1 flex-col justify-center p-4 text-center space-y-4">
      <h1 class="heading-styles">Server error</h1>
      <p>500 internal error: {(props.error as Error).message}</p>
      <p>
        <a href="/" class="link-styles">Return home &#8250;</a>
      </p>
    </main>
  );
}

``````


### routes/user-profile.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/user-profile.tsx
`relative_path`: routes/user-profile.tsx
`format`: Arbitrary Binary Data
`size`: 7191   




### routes/account/index.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/account/index.tsx
`relative_path`: routes/account/index.tsx
`format`: Arbitrary Binary Data
`size`: 2465   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { defineRoute } from "$fresh/server.ts";
import type { SignedInState } from "@/plugins/session.ts";
import { isStripeEnabled } from "@/utils/stripe.ts";
import Head from "@/components/Head.tsx";
import GitHubAvatarImg from "@/components/GitHubAvatarImg.tsx";
import { PremiumBadge } from "@/components/PremiumBadge.tsx";

export default defineRoute<SignedInState>((_req, ctx) => {
  const { sessionUser } = ctx.state;
  const action = sessionUser.isSubscribed ? "Manage" : "Upgrade";

  return (
    <>
      <Head title="Account" href={ctx.url.href} />
      <main class="max-w-lg m-auto w-full flex-1 p-4 flex flex-col justify-center gap-8">
        {sessionUser.picture
          ? (
            <img
              src={sessionUser.picture}
              alt="Profile Picture"
              class="mx-auto rounded-full w-24 h-24"
            />
          )
          : (
            <GitHubAvatarImg
              login={sessionUser.login}
              size={240}
              class="mx-auto"
            />
          )}
        <ul class="space-y-4">
          <li>
            <strong>Username</strong>
            <p class="flex flex-wrap justify-between">
              <span>
                {sessionUser.name || sessionUser.login}
              </span>
              <a href={`/users/${sessionUser.login}`} class="link-styles">
                Go to my profile &#8250;
              </a>
            </p>
          </li>
          <li>
            <strong>Subscription</strong>
            <p class="flex flex-wrap justify-between">
              <span>
                {sessionUser.isSubscribed
                  ? (
                    <>
                      Premium <PremiumBadge class="size-5 inline" />
                    </>
                  )
                  : (
                    "Free"
                  )}
              </span>
              {isStripeEnabled() && (
                <span>
                  <a
                    class="link-styles"
                    href={`/account/${action.toLowerCase()}`}
                  >
                    {action} &#8250;
                  </a>
                </span>
              )}
            </p>
          </li>
        </ul>
        <a
          href="/signout?success_url=/"
          class="button-styles block text-center"
        >
          Sign out
        </a>
      </main>
    </>
  );
});

``````


### routes/account/manage.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/account/manage.ts
`relative_path`: routes/account/manage.ts
`format`: Arbitrary Binary Data
`size`: 688   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { defineRoute } from "$fresh/server.ts";
import type { SignedInState } from "@/plugins/session.ts";
import { redirect } from "@/utils/http.ts";
import { isStripeEnabled, stripe } from "@/utils/stripe.ts";

export default defineRoute<SignedInState>(async (_req, ctx) => {
  const { sessionUser } = ctx.state;
  if (!isStripeEnabled() || sessionUser.stripeCustomerId === undefined) {
    return ctx.renderNotFound();
  }

  const { url } = await stripe.billingPortal.sessions.create({
    customer: sessionUser.stripeCustomerId,
    return_url: ctx.url.origin + "/account",
  });
  return redirect(url);
});

``````


### routes/account/upgrade.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/account/upgrade.ts
`relative_path`: routes/account/upgrade.ts
`format`: Arbitrary Binary Data
`size`: 1032   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { defineRoute } from "$fresh/server.ts";
import type { SignedInState } from "@/plugins/session.ts";
import { redirect } from "@/utils/http.ts";
import {
  getStripePremiumPlanPriceId,
  isStripeEnabled,
  stripe,
} from "@/utils/stripe.ts";

export default defineRoute<SignedInState>(async (_req, ctx) => {
  if (!isStripeEnabled()) return ctx.renderNotFound();
  const stripePremiumPlanPriceId = getStripePremiumPlanPriceId();
  if (stripePremiumPlanPriceId === undefined) {
    throw new Error(
      '"STRIPE_PREMIUM_PLAN_PRICE_ID" environment variable not set',
    );
  }

  const { url } = await stripe.checkout.sessions.create({
    success_url: ctx.url.origin + "/account",
    customer: ctx.state.sessionUser.stripeCustomerId,
    line_items: [
      {
        price: stripePremiumPlanPriceId,
        quantity: 1,
      },
    ],
    mode: "subscription",
  });
  if (url === null) return ctx.renderNotFound();

  return redirect(url);
});

``````


### routes/welcome.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/welcome.tsx
`relative_path`: routes/welcome.tsx
`format`: Arbitrary Binary Data
`size`: 1847   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { defineRoute } from "$fresh/src/server/defines.ts";
import Head from "@/components/Head.tsx";
import { isGitHubSetup } from "@/utils/github.ts";

function SetupInstruction() {
  return (
    <div class="bg-green-50 dark:bg-gray-900 dark:border dark:border-green-800 rounded-xl max-w-screen-sm mx-auto p-8 space-y-2">
      <h1 class="text-2xl font-medium">Welcome to SaaSKit!</h1>

      <p class="text-gray-600 dark:text-gray-400">
        To enable user login, you need to configure the GitHub OAuth application
        and set environment variables.
      </p>

      <p>
        <a
          href="https://github.com/denoland/saaskit#get-started-locally"
          class="inline-flex gap-2 text-green-600 dark:text-green-400 hover:underline cursor-pointer"
        >
          Get started locally guide &#8250;
        </a>
      </p>
      <p>
        <a
          href="https://github.com/denoland/saaskit#deploy-to-production"
          class="inline-flex gap-2 text-green-600 dark:text-green-400 hover:underline cursor-pointer"
        >
          Deploy to production guide &#8250;
        </a>
      </p>

      <p class="text-gray-600 dark:text-gray-400">
        After setting up{" "}
        <span class="bg-green-100 dark:bg-gray-800 p-1 rounded">
          GITHUB_CLIENT_ID
        </span>{" "}
        and{" "}
        <span class="bg-green-100 dark:bg-gray-800 p-1 rounded">
          GITHUB_CLIENT_SECRET
        </span>
        , this message will disappear.
      </p>
    </div>
  );
}

export default defineRoute((_req, ctx) => {
  return (
    <>
      <Head title="Welcome" href={ctx.url.href} />
      <main class="flex-1 flex justify-center items-center">
        {!isGitHubSetup() && <SetupInstruction />}
      </main>
    </>
  );
});

``````


### routes/therapy/index.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/therapy/index.tsx
`relative_path`: routes/therapy/index.tsx
`format`: Arbitrary Binary Data
`size`: 262   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import { redirect } from "@/utils/http.ts";
import { Handlers } from "$fresh/server.ts";

export const handler: Handlers = {
  GET() {
    return redirect("/therapy/relaxation");
  },
};

``````


### routes/therapy/[therapyId].tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/therapy/[therapyId].tsx
`relative_path`: routes/therapy/[therapyId].tsx
`format`: Arbitrary Binary Data
`size`: 5026   


``````
import { defineRoute } from "$fresh/server.ts";
import { ArrowLeft, Plus } from "lucide-preact";
import { TherapyCard } from "@/components/TherapyCard.tsx";
import { ColorPicker } from "@/components/ColorPicker.tsx";
import { TemperatureRange } from "@/components/TemperatureRange.tsx";
import { therapyOptions } from "@/data/therapies.ts";

export default defineRoute((_req, ctx) => {
  const { therapyId } = ctx.params;

  const customTherapy = {
    lightColor: "#3B82F6",
    temperatureRange: { min: 15, max: 25 },
    description: "Custom therapy configuration",
    duration: 20,
  };

  const handleCustomTherapySubmit = () => {
    console.log("Custom therapy started with:", customTherapy);
  };

  return (
    <div class="min-h-screen bg-gradient-to-br from-indigo-50 to-purple-50 py-4 sm:py-8 px-2 sm:px-4 lg:px-8">
      <div class="max-w-4xl mx-auto">
        <div class="flex items-center gap-4 mb-4 sm:mb-8">
          <a
            href="/"
            class="p-2 rounded-full hover:bg-white/50 transition-colors"
          >
            <ArrowLeft class="w-5 h-5 sm:w-6 sm:h-6 text-gray-600" />
          </a>
          <h1 class="text-2xl sm:text-3xl font-bold text-gray-900">
            {therapyId === "create"
              ? "Create Custom Therapy"
              : "Select Therapy"}
          </h1>
        </div>

        {therapyId === "create" ? (
          <div class="bg-white rounded-xl shadow-lg p-4 sm:p-6 space-y-4 sm:space-y-6">
            <div class="space-y-3 sm:space-y-4">
              <h2 class="text-lg sm:text-xl font-semibold text-gray-900">
                Light Color
              </h2>
              <ColorPicker
                value={customTherapy.lightColor}
                onChange={(color) => (customTherapy.lightColor = color)}
              />
            </div>

            <div class="space-y-3 sm:space-y-4">
              <h2 class="text-lg sm:text-xl font-semibold text-gray-900">
                Temperature Range
              </h2>
              <TemperatureRange
                min={customTherapy.temperatureRange.min}
                max={customTherapy.temperatureRange.max}
                onChange={(min, max) =>
                  (customTherapy.temperatureRange = { min, max })
                }
              />
            </div>

            <div class="space-y-3 sm:space-y-4">
              <h2 class="text-lg sm:text-xl font-semibold text-gray-900">
                Session Duration
              </h2>
              <div class="flex items-center gap-4">
                <input
                  type="range"
                  min="5"
                  max="45"
                  step="5"
                  value={customTherapy.duration}
                  // deno-lint-ignore fresh-server-event-handlers
                  onInput={(e) =>
                    (customTherapy.duration = Number(e.currentTarget.value))
                  }
                  class="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
                />
                <span class="w-16 sm:w-20 text-right">
                  {customTherapy.duration} min
                </span>
              </div>
            </div>

            <button
              // deno-lint-ignore fresh-server-event-handlers
              onClick={handleCustomTherapySubmit}
              class="w-full py-2 sm:py-3 px-4 bg-indigo-600 text-white rounded-lg font-medium hover:bg-indigo-700 transition-colors"
            >
              Start Custom Therapy
            </button>
          </div>
        ) : (
          <div class="space-y-4 sm:space-y-6">
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
              {therapyOptions.map((therapy) => (
                <TherapyCard
                  key={therapy.id}
                  therapy={therapy}
                  selected={therapy.id === therapyId}
                  onSelect={() => console.log("Therapy selected:", therapy)}
                />
              ))}
            </div>

            <div class="flex justify-center">
              <a
                href="/therapy/create"
                class="flex items-center gap-2 py-2 px-4 text-gray-600 hover:text-gray-900 transition-colors"
              >
                <Plus class="w-5 h-5" />
                Create Custom Therapy
              </a>
            </div>

            {/* Start Therapy Button */}
            <div class="flex justify-center mt-6">
              <a href="/therapy-progress">
                <button
                  class={`py-2 px-4 bg-indigo-600 text-white rounded-lg font-medium transition-colors ${
                    therapyId
                      ? "hover:bg-indigo-700"
                      : "opacity-50 cursor-not-allowed"
                  }`}
                  // disabled={!selectedTherapy}
                >
                  Start Selected Therapy
                </button>
              </a>
            </div>
          </div>
        )}
      </div>
    </div>
  );
});

``````


### routes/_404.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/_404.tsx
`relative_path`: routes/_404.tsx
`format`: Arbitrary Binary Data
`size`: 356   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.

export default function NotFoundPage() {
  return (
    <main class="flex-1 p-4 flex flex-col justify-center text-center">
      <h1 class="heading-styles">Page not found</h1>
      <p>
        <a href="/" class="link-styles">Return home &#8250;</a>
      </p>
    </main>
  );
}

``````


### routes/machine-control.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/machine-control.tsx
`relative_path`: routes/machine-control.tsx
`format`: Arbitrary Binary Data
`size`: 172   


``````
import { defineRoute } from "$fresh/server.ts";

export default defineRoute((_req, ctx) => {
  return (
    <div>
      <h1>Machine Control Screen</h1>
    </div>
  );
});

``````


### routes/_app.tsx
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/routes/_app.tsx
`relative_path`: routes/_app.tsx
`format`: Arbitrary Binary Data
`size`: 1195   


``````
// Copyright 2023-2024 the Deno authors. All rights reserved. MIT license.
import Header from "@/components/Header.tsx";
// import Footer from "@/components/Footer.tsx";
import type { State } from "@/plugins/session.ts";
import { defineApp } from "$fresh/server.ts";
import { redirect } from "@/utils/http.ts";

const protectedRoutes = ["/dashboard", "/profile", "/settings"];

export default defineApp<State>((req, ctx) => {
  const isSignedIn = ctx.state.sessionUser !== undefined;
  const currentPath = new URL(req.url).pathname;

  if (!isSignedIn && protectedRoutes.includes(currentPath)) {
    return redirect("/login");
  }

  return (
    <html lang="en">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <link rel="stylesheet" href="/styles.css" />
      </head>
      <body>
        <div class="dark:bg-gray-900">
          <div class="flex flex-col min-h-screen mx-auto  w-full dark:text-white">
            <Header url={ctx.url} sessionUser={ctx.state?.sessionUser} />
            <ctx.Component />
            {/* <Footer /> */}
          </div>
        </div>
      </body>
    </html>
  );
});

``````


### e2e_test.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-platform/e2e_test.ts
`relative_path`: e2e_test.ts
`format`: Arbitrary Binary Data
`size`: 28257   





## vacuul
`clone_url`: https://github.com/vacuul-dev/vacuul.git


### main.typ
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/main.typ
`relative_path`: main.typ
`format`: Arbitrary Binary Data
`size`: 11193   




### refs.bib
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/refs.bib
`relative_path`: refs.bib
`format`: Arbitrary Binary Data
`size`: 331   


``````
@misc{wikipedia_iosevka,
  title = {Iosevka},
  year = 2024,
  month = mar,
  journal = {Wikipedia},
  url = {https://en.wikipedia.org/w/index.php?title=Iosevka&oldid=1217127968},
  urldate = {2024-06-18},
  copyright = {Creative Commons Attribution-ShareAlike License},
  note = {Page Version ID: 1217127968},
  language = {en}
}

``````


### raw-info.txt
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/raw-info.txt
`relative_path`: raw-info.txt
`format`: Arbitrary Binary Data
`size`: 175   


``````
ssh -i "vacuul-server-key.pem" ec2-user@3.70.228.144

mysql -h vacuul-database-1.cluster-csmkujozbvza.eu-central-1.rds.amazonaws.com -u vacuuladmin -p

/home/ec2-user/Backend2
``````


### main.pdf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/main.pdf
`relative_path`: main.pdf
`format`: Portable Document Format
`size`: 60964   




### SUMMARY.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/SUMMARY.md
`relative_path`: SUMMARY.md
`format`: Arbitrary Binary Data
`size`: 4283   


``````
# Vacuul Technical Documentation - November 2024

## 1. Frontend (Flutter)

### 1.1. App Structure and Navigation
* **Main Entry Points**
  * `main.dart`: Entry point, Firebase initialization, API credentials management
  * `index.dart`: Barrel file for page exports
  * `flutter_flow/nav/nav.dart`: GoRouter navigation setup
  * `lib/app_state.dart`: Global app state management

### 1.2. Authentication
* **Core Components**
  * Firebase Authentication integration
  * Custom user provider (`VacuulFirebaseUser`)
  * `FirebaseAuthManager` implementation
  * Authentication utilities in `auth_util.dart`

### 1.3. Backend Integration
* **Core Files**
  * `backend.dart`: Firestore functions and queries
  * `api.dart`: Global API variables
  * `api_requests/api_calls.dart`: API call definitions
  * `api_requests/api_manager.dart`: API request handling
  * `api_requests/get_streamed_response.dart`: Streaming utilities
* **External Services**
  * Firebase configuration
  * Firebase Storage integration
  * Gemini AI integration

### 1.4. UI Components
* **Custom Components**
  * `premium_content_toast_widget.dart`: Treatment info display
* **FlutterFlow Components**
  * Animation utilities
  * Calendar integration
  * Dropdown menus
  * Google Maps integration
  * Icon buttons
  * Language selector
  * Theme management
  * Timer functionality
  * Form controllers
  * Internationalization support
  * File handling
  * Custom functions

### 1.5. Screen Architecture
#### 1.5.1. Main Screens
* Landing Screen
* Onboarding Screen
* Home Screen
* News Screen
* Profile Screen

#### 1.5.2. Authentication Screens
* Login Screen
* Register Screen
* Forgot Password Screen
* Profile Creation/Edit Screens

#### 1.5.3. Booking Flow
* Calendar Screen
* Location Screen
* Booking Screen
* Purchasing Screen
* Success Booking Screen

#### 1.5.4. Payment Screens
* Payment Details Screen
* Add Payment Screen
* Payment Success Screen
* Credit Card Edit Screen

#### 1.5.5. Therapy Screens
* Choose Therapy Screen
* Create Therapy Screen
* Treatment Start Screen
* Active Treatment Screen
* End Treatment Screen
* Treatment Rating Screen

#### 1.5.6. Support Screens
* Agreement Screen
* Terms & Conditions Screen
* Support Details Screen
* Contact Screens (Call, Email, Chat)

#### 1.5.7. Error Handling
* Error Payment Screen
* Generic Error Screen
* Loading Screens

## 2. Backend (Node.js)

### 2.1. Core Structure
* **Server Setup**
  * `app.js`: Express configuration
  * `server.js`: Server initialization
  * Environment configuration
  * Package management

### 2.2. API Routes
* **Authentication Routes**
  * User registration
  * Login functionality
* **Machine Routes**
  * Machine management
  * Status updates
* **Payment Routes**
  * Payment processing
* **Session Routes**
  * Booking management
  * Status tracking
* **Treatment Routes**
  * Treatment creation
  * Activation handling

### 2.3. Controllers
* Authentication Controller
* Machine Controller
* Payment Controller
* Session Controller
* Treatment Controller

### 2.4. Data Models
#### 2.4.1. User Model
* Account management
* User lookup functionality

#### 2.4.2. Machine Model
* Machine data handling
* Status management

#### 2.4.3. Session Model
* Booking management
* User session tracking

#### 2.4.4. Treatment Model
* Treatment lifecycle management
* Progress tracking
* Activation control

#### 2.4.5. Payment Model
* Payment record management

#### 2.4.6. Error Model
* Machine error logging
* Diagnostic data capture

### 2.5. External Services
* MySQL database integration
* Stripe payment processing
* Winston logging system

### 2.6. API Implementation Status
#### 2.6.1. Fully Implemented Endpoints
* Authentication endpoints
* Basic machine management
* Payment processing
* Session booking
* Treatment management

#### 2.6.2. Partially Implemented/Missing Endpoints
* Machine start functionality
* Progress tracking
* Settings management
* Upcoming treatments
* Time slot management

### 2.7. Security Considerations
* JWT implementation
* Authentication flow
* Authorization controls
* Data validation
* Error handling

### 2.8. Improvement Recommendations
* Security enhancements
* Missing endpoint implementation
* Error handling optimization
* Input validation
* Documentation updates
``````


### PROPOSAL_TLDR.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/PROPOSAL_TLDR.md
`relative_path`: PROPOSAL_TLDR.md
`format`: Arbitrary Binary Data
`size`: 2171   


``````
# Vacuul Project Proposal

## Current State

The **Vacuul** project currently features a Flutter frontend and a Node.js backend that handles user authentication, booking, payment, and treatment processes. The backend uses Firebase and Stripe to provide support, but these services present limitations in terms of cost, scalability, and feature flexibility. Therefore, a more adaptable solution is required to meet the project's evolving needs.

## Proposal

We propose transitioning to **AppWrite** to accelerate the completion of the backend and facilitate a more efficient integration with the Flutter frontend.

## Why AppWrite?

Moving to **AppWrite** offers several key advantages:

1. **Integrated Backend**: AppWrite combines essential services such as database management, authentication, storage, and real-time capabilities, simplifying the overall architecture.
2. **Customizable Functions**: It provides more control over backend logic, making customization easier and more effective.
3. **Unified API**: A unified API streamlines team collaboration and reduces development complexity.
4. **Cost Efficiency**: Self-hosting AppWrite can significantly lower operational costs compared to Firebase's pay-as-you-go model.

## Development Strategy

To ensure a smooth transition, we have outlined a four-phase strategy:

1. **Phase 1**: Replace Firebase with AppWrite for user authentication, ensuring a seamless switch for all existing users.
2. **Phase 2**: Implement core APIs using Deno and TypeScript within the AppWrite environment, focusing on user, booking, and treatment functionalities.
3. **Phase 3**: Integrate the newly developed backend APIs with the Flutter frontend to enhance user profile management, booking, and payment workflows.
4. **Phase 4**: Update firmware to ensure compatibility with AppWrite, replacing WebSocket logic where necessary to achieve better stability and integration.

## Conclusion

Transitioning to **AppWrite** will improve the scalability, maintainability, and cost-effectiveness of the **Vacuul** project. This strategic shift will not only enhance the user experience but also provide the foundation for future growth.

``````


### CURRENT_STATE_RAW.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/CURRENT_STATE_RAW.md
`relative_path`: CURRENT_STATE_RAW.md
`format`: Arbitrary Binary Data
`size`: 38864   




### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 9   


``````
# vacuul

``````


### gantt.typ
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/gantt.typ
`relative_path`: gantt.typ
`format`: Arbitrary Binary Data
`size`: 2414   


``````
#import "@preview/timeliney:0.1.0"

#timeliney.timeline(
  show-grid: true,
  {
    import timeliney: *

    headerline(group(([*2024*], 7), ([*2025*], 3)))
    headerline(group(([*November*], 3), ([*December*], 4), ([*January*], 3)))
    
    taskgroup(title: [*Framework Setup*], {
      task("Deno SaaSKit Environment Setup", (0, 1), style: (stroke: 2pt + gray))
      task("GitHub Repository and Configurations", (0, 1), style: (stroke: 2pt + gray))
      task("Initial Third-Party Service Configuration", (0.5, 1.5), style: (stroke: 2pt + gray))
    })

    taskgroup(title: [*Feature Development (Core)*], {
      task("Hardware-Software Connection Setup", (1, 4), style: (stroke: 2pt + gray))
      task("Purchase and Reservation Platform", (2, 5), style: (stroke: 2pt + gray))
      task("User Management and Authentication", (1.5, 3), style: (stroke: 2pt + gray))
    })

    taskgroup(title: [*Integration (Payments & Device)*], {
      task("Stripe Integration for Payment Processing", (4, 5), style: (stroke: 2pt + gray))
      task("Device Control Integration", (4, 5.5), style: (stroke: 2pt + gray))
    })

    taskgroup(title: [*Testing & Quality Assurance*], {
      task("Unit Testing for Core Features", (5, 6), style: (stroke: 2pt + gray))
      task("Integration Testing for Payments and Device", (5.5, 7), style: (stroke: 2pt + gray))
      task("User Acceptance Testing", (6.5, 8), style: (stroke: 2pt + gray))
    })

    taskgroup(title: [*Deployment*], {
      task("Initial Deployment", (1.5, 1.75), style: (stroke: 2pt + gray))
      task("Midpoint Deployment", (4, 4.25), style: (stroke: 2pt + gray))
      task("Testing Deployment", (6, 6.25), style: (stroke: 2pt + gray))
      task("Final Deployment on Deno Deploy", (8, 8.5), style: (stroke: 2pt + gray))
      task("Final Configuration of Third-Party Services", (8, 8.5), style: (stroke: 2pt + gray))
    })

    taskgroup(title: [*Adjustments & Changes*], {
      task("Feedback-Based Changes and Adjustments", (8.5, 10), style: (stroke: 2pt + gray))
    })
    milestone(
      at: 3,
      style: (stroke: (dash: "dashed")),
      align(center, [])
    )
    
    milestone(
      at: 7,
      style: (stroke: (dash: "dashed")),
      align(center, [])
    )

    milestone(
      at: 10,
      style: (stroke: (dash: "solid")),
      align(center, [
        *Project Completion*\
        Mid-January 2025
      ])
    )
  }
)

``````


### new_gantt.pdf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/new_gantt.pdf
`relative_path`: new_gantt.pdf
`format`: Portable Document Format
`size`: 19850   




### system_architecture_raw.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/system_architecture_raw.md
`relative_path`: system_architecture_raw.md
`format`: Arbitrary Binary Data
`size`: 6019   




### new_gantt.typ
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/new_gantt.typ
`relative_path`: new_gantt.typ
`format`: Arbitrary Binary Data
`size`: 2892   


``````
#import "@preview/timeliney:0.1.0"

#timeliney.timeline(
  show-grid: true,
  {
    import timeliney: *

    headerline(group(([*2024*], 5), ([*2025*], 4)))
    headerline(group(([*Nov*], 1), ([*December*], 4), ([*January*], 4)))

    taskgroup(title: [*Phase 1: Flutter Auth Integration*], {
      task("AppWrite SDK Setup", (0, 1), style: (stroke: 2pt + gray))
      task("Login and Registration Flows", (0.5, 1.5), style: (stroke: 2pt + gray))
      task("Password Recovery Flow", (1, 1.5), style: (stroke: 2pt + gray))
      task("Session Management", (1.25, 2), style: (stroke: 2pt + gray))
      task("Testing Authentication", (1.5, 2), style: (stroke: 2pt + gray))
    })

    taskgroup(title: [*Phase 2: Backend API Development*], {
      task("Deno + AppWrite Functions Setup", (0, 2), style: (stroke: 2pt + gray))
      task("Core API Development (User, Machine, Treatments)", (1, 4), style: (stroke: 2pt + gray))
      task("Payment Flow Integration (Stripe)", (3, 4), style: (stroke: 2pt + gray))
      // task("API Contracts Documentation", (3, 4), style: (stroke: 2pt + gray))
      task("Backend Unit Testing", (3, 4), style: (stroke: 2pt + gray))
    })

    taskgroup(title: [*Phase 3: Full Flutter Integration*], {
      task("User Profile Management", (2, 3), style: (stroke: 2pt + gray))
      task("Booking and Treatment Flows", (3, 4), style: (stroke: 2pt + gray))
      task("Real-Time Updates for Machines & Treatments", (3, 5), style: (stroke: 2pt + gray))
      task("Payment Workflow Integration", (4, 5), style: (stroke: 2pt + gray))
      task("Testing Frontend Integration", (4, 5), style: (stroke: 2pt + gray))
    })

    taskgroup(title: [*Phase 4: Device Integration and Firmware Evaluation*], {
      task("Evaluate Current Firmware", (2, 3), style: (stroke: 2pt + gray))
      task("Ensure HTTP Communication with Backend", (2, 6), style: (stroke: 2pt + gray))
      task("Optimize Firmware Logic (WebSocket/HTTP)", (5, 6), style: (stroke: 2pt + gray))
      task("Test Device Communication", (5, 7), style: (stroke: 2pt + gray))
      task("Real-World Device Testing", (2, 7), style: (stroke: 2pt + gray))
    })

    taskgroup(title: [*Final Adjustments and Deployment*], {
      task("Feedback-Based Adjustments", (7, 9), style: (stroke: 2pt + gray))
      task("Final Testing and Deployment", (8, 9), style: (stroke: 2pt + gray))
    })

    milestone(
      at: 1,
      style: (stroke: (dash: "dashed")),
      align(center, [])
    )
    milestone(
      at: 5,
      style: (stroke: (dash: "dashed")),
      align(center, [])
    )
    
    // milestone(
    //   at: 7.5,
    //   style: (stroke: (dash: "dashed")),
    //   align(center, [])
    // )

    // milestone(
    //   at: 10.5,
    //   style: (stroke: (dash: "solid")),
    //   align(center, [
    //     *Project Completion*\
    //     Late-January 2025
    //   ])
    // )
  }
)

``````


### PROPOSAL.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul/PROPOSAL.md
`relative_path`: PROPOSAL.md
`format`: Arbitrary Binary Data
`size`: 5757   





## vacuul-firmware
`clone_url`: https://github.com/vacuul-dev/vacuul-firmware.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-firmware/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 3972   


``````
# API System Documentation

This repository contains the implementation of an API system designed for managing asynchronous communication with a remote service. It is a modular system that supports multiple endpoints, thread-safe operations, and a state machine to handle the lifecycle of API requests.

---

## Table of Contents
1. [Overview](#overview)
2. [Endpoints](#endpoints)
3. [Workflow](#workflow)
4. [Dependencies](#dependencies)
5. [Contributing](#contributing)

---

## Overview

The API system is designed to:
- Manage HTTP requests for different endpoints.
- Parse and validate responses.
- Maintain the state of requests and responses in a thread-safe manner.

It uses:
- A **state machine** to control the lifecycle of requests and responses.
- Thread-safe mechanisms for shared data handling.
- JSON serialization/deserialization for request and response data.

---

## Endpoints

| **Endpoint**         | **Method** | **Input Struct**      | **Output Struct**      |
|-----------------------|------------|-----------------------|------------------------|
| `/api/machines`       | POST       | `CommissionReq`       | `CommissionRes`        |
| `/api/machines/start` | POST       | `StartReq`            | `StartRes`             |
| `/api/machines/settings` | GET     | `SettingsReq`         | `SettingsRes`          |
| `/api/machines/progress` | POST    | `ProgressReq`         | `ProgressRes`          |
| `/api/machines/error` | POST       | `ErrorReq`            | *(No specific output)* |

### Details
1. **Commission**  
   - **Description:** Registers a machine.  
   - **Input Struct:** `CommissionReq` (contains `commissionId`).  
   - **Output Struct:** `CommissionRes` (contains `machineId`, `timezone`).  

2. **Start**  
   - **Description:** Authorizes treatment to start.  
   - **Input Struct:** `StartReq` (inherits from `req_data_base`).  
   - **Output Struct:** `StartRes` (contains `treatClearance` boolean).  

3. **Settings**  
   - **Description:** Retrieves machine/user configuration settings.  
   - **Input Struct:** `SettingsReq` (contains `machineId`).  
   - **Output Struct:** `SettingsRes` (contains user information and treatment configuration).  

4. **Progress**  
   - **Description:** Reports treatment progress or data.  
   - **Input Struct:** `ProgressReq` (contains `appData`).  
   - **Output Struct:** `ProgressRes` (contains `abortTreatment` boolean).  

5. **Error**  
   - **Description:** Handles error reporting.  
   - **Input Struct:** `ErrorReq` (contains `status` and `appData`).  
   - **Output Struct:** *(No structured output expected)*.  

---

## Workflow

1. **Request Handling**:
   - Requests are dynamically built based on the endpoint and its input struct.
   - Each request is serialized into a JSON body for transmission.

2. **State Management**:
   - The state machine transitions between states (`idle`, `req`, `res`, `error`) to handle requests and responses.

3. **Response Parsing**:
   - Each response is parsed and validated to ensure correctness.
   - Parsed data is stored in the shared data structure for access by other components.

4. **Thread Operation**:
   - A dedicated thread manages the lifecycle of API calls and synchronizes shared data access.

---

## Dependencies

This project uses the following dependencies:
- **C++ Standard Library** for threading, JSON handling, and string manipulation.
- **Custom Middleware** for logging, curl operations, and utility functions.

---

## Contributing

Contributions are welcome! Please follow these steps:
1. Fork the repository.
2. Create a new branch (`git checkout -b feature-branch`).
3. Commit your changes (`git commit -m 'Add new feature'`).
4. Push to the branch (`git push origin feature-branch`).
5. Open a pull request.

For major changes, please open an issue first to discuss your proposal.

---

## License

This project is licensed under the terms of the copyright holders. See the source files for details.

``````


### src/api.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-firmware/src/api.cpp
`relative_path`: src/api.cpp
`format`: Arbitrary Binary Data
`size`: 16797   




### src/data.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-firmware/src/data.h
`relative_path`: src/data.h
`format`: Arbitrary Binary Data
`size`: 3882   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_API_DATA_H
#define IG_API_DATA_H

#include <cstddef>
#include <cstdint>
#include <string>

#include "application/treatmentData.h"

namespace api {

class req_data_base {
 public:
  req_data_base() {}
  virtual ~req_data_base() {}

  // aka serialise
  virtual std::string httpBody() const;
};

class res_data_base {
 public:
  res_data_base() {}
  virtual ~res_data_base() {}

  virtual void set(const std::string& httpBody) noexcept(false) = 0;
};

//======================================================================================================================
// requests

class CommissionReq
    : public req_data_base  // TODO is a derived class really needed? comission
                            // id is already in appdata in the default req JSON
{
 public:
  class ID {
   public:
    ID() = delete;
    explicit ID(const std::string& id) : m_id(id) {}

    const std::string& get() const { return m_id; }

   private:
    std::string m_id;
  };

 public:
  CommissionReq() : req_data_base(), m_id() {}

  CommissionReq(const ID& id) : req_data_base(), m_id(id.get()) {}

  virtual ~CommissionReq() {}

  virtual std::string httpBody() const;

 private:
  std::string m_id;
};

using StartReq = req_data_base;

class SettingsReq : public req_data_base {
 public:
  SettingsReq();

  virtual ~SettingsReq() {}

  const std::string& machineIdStr() const { return m_machineIdStr; }

 private:
  std::string m_machineIdStr;

  virtual std::string httpBody() const { return ""; }
};

class ProgressReq : public req_data_base {
 public:
  ProgressReq() : req_data_base() {}

  virtual ~ProgressReq() {}

  virtual std::string httpBody() const;
};

class ErrorReq : public req_data_base {
 public:
  ErrorReq() : req_data_base() {}

  virtual ~ErrorReq() {}

  virtual std::string httpBody() const;
};

//======================================================================================================================
// responses

class CommissionRes : public res_data_base {
 public:
  CommissionRes()
      : res_data_base(),
        m_dataReady(false),
        m_machineId("0"),
        m_timezone("Europe/Zurich")  // allways init with a valid tz, e.g.
                                     // "Europe/London"
  {}

  virtual ~CommissionRes() {}

  bool dataReady() const { return m_dataReady; }

  const std::string& machineId() const { return m_machineId; }
  const std::string& timezone() const { return m_timezone; }

  virtual void set(const std::string& httpBody) noexcept(false);

 private:
  bool m_dataReady;
  std::string m_machineId;
  std::string m_timezone;
};

class StartRes : public res_data_base {
 public:
  StartRes() : res_data_base(), m_treatClearance(false) {}

  virtual ~StartRes() {}

  bool treatClearance() const { return m_treatClearance; }

  virtual void set(const std::string& httpBody) noexcept(false);

 private:
  bool m_treatClearance;
};

class SettingsRes : public res_data_base {
 public:
  SettingsRes()
      : res_data_base(),
        m_treatUser("<nickname>"),
        m_treatConfig(app::treat::Config::blocks_type()) {}

  virtual ~SettingsRes() {}

  const app::treat::User& user() const { return m_treatUser; }
  const app::treat::Config& treatConfig() const { return m_treatConfig; }

  virtual void set(const std::string& httpBody) noexcept(false);

 private:
  app::treat::User m_treatUser;
  app::treat::Config m_treatConfig;
  // m_facrotyReset m_localReset
};

class ProgressRes : public res_data_base {
 public:
  ProgressRes() : res_data_base(), m_abort(false) {}

  virtual ~ProgressRes() {}

  bool abortTreatment() const { return m_abort; }

  virtual void set(const std::string& httpBody) noexcept(false);

 private:
  bool m_abort;
};

}  // namespace api

#endif  // IG_API_DATA_H

``````


### src/data.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-firmware/src/data.cpp
`relative_path`: src/data.cpp
`format`: Arbitrary Binary Data
`size`: 7270   




### src/api.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-firmware/src/api.h
`relative_path`: src/api.h
`format`: Arbitrary Binary Data
`size`: 4438   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

/*

data:
 - set req data (one per endpoint)
 - get res data (one per endpoint)
 - status
    - state (idle, req, res, error)
    - current endpoint (don't care in idle state)
    - timestamps (req, res)
    - duration

Serialising of the req data and parsing of the res data is explicitly done in
the API thread.

controlling / thread sync:
 - set req data triggers the request, returns with no effect if state != idle
 - get res data sets state to idle
 - flush method to set state to idle (in case of error), returns with no effect
if state is neither error nor res

*/

#ifndef IG_API_API_H
#define IG_API_API_H

#include <cstddef>
#include <cstdint>
#include <ctime>
#include <string>

#include "api/data.h"
#include "middleware/thread.h"
#include "middleware/util.h"

namespace api::thread {

typedef enum {
  ep_commission,
  ep_start,     // start treatment clearance
  ep_settings,  // treatment/user config
  ep_progress,  // report treatment progress/data
  ep_error,

  ep__end_
} endpoint_t;

typedef enum STATE {
  state_idle,   // ready to make a request
  state_req,    // request is ongoing
  state_res,    // response is ready
  state_error,  // an error occured

  state__end_
} state_t;

class Status {
 public:
  Status() : m_state(state_idle), m_ep(), m_tReq(), m_tRes(), m_dur(0) {}

  virtual ~Status() {}

  int reqEndpoint(endpoint_t ep);
  void resGotten(endpoint_t ep);
  void setState(state_t state) { m_state = state; }
  void setTReq(time_t t) { m_tReq = t; }
  void setTRes(time_t t) { m_tRes = t; }
  void setDuration(omw_::clock::timepoint_t dur_us) { m_dur = dur_us; }

  state_t state() const { return m_state; }
  endpoint_t endpoint() const { return m_ep; }
  time_t tReq() const { return m_tReq; }
  time_t tRes() const { return m_tRes; }
  omw_::clock::timepoint_t duration() const { return m_dur; }

 private:
  state_t m_state;
  endpoint_t m_ep;
  time_t m_tReq;
  time_t m_tRes;
  omw_::clock::timepoint_t m_dur;
};

std::string toString(endpoint_t ep);

class ThreadSharedData : public ::thread::SharedData {
 public:
  ThreadSharedData() : m_status() {}

  virtual ~ThreadSharedData() {}

  // extern
 public:
  int reqCommission(const CommissionReq& data);
  int reqStart(const StartReq& data);
  int reqSettings(const SettingsReq& data);
  int reqProgress(const ProgressReq& data);
  int reqError(const ErrorReq& data);

  CommissionRes getCommissionRes() const;
  StartRes getStartRes() const;
  SettingsRes getSettingsRes() const;
  ProgressRes getProgressRes() const;

  void flush() const;

  // clang-format off
    Status status() const { lock_guard lg(m_mtx); return m_status; }
  // clang-format on

  // intern
 public:
  // clang-format off
    void setState(state_t state)    { lock_guard lg(m_mtx); m_status.setState(state); }
    void setTReq(time_t t)          { lock_guard lg(m_mtx); m_status.setTReq(t); }
    void setTRes(time_t t)          { lock_guard lg(m_mtx); m_status.setTRes(t); }
    void setDur(omw_::clock::timepoint_t dur_us) { lock_guard lg(m_mtx); m_status.setDuration(dur_us); }

    void setCommissionRes(const CommissionRes& data) { lock_guard lg(m_mtx); m_commissionRes = data; }
    void setStartRes(const StartRes& data) { lock_guard lg(m_mtx); m_startRes = data; }
    void setSettingsRes(const SettingsRes& data) { lock_guard lg(m_mtx); m_settingsRes = data; }
    void setProgressRes(const ProgressRes& data) { lock_guard lg(m_mtx); m_progressRes = data; }

    CommissionReq getCommissionReq() const { lock_guard lg(m_mtx); return m_commissionReq; }
    StartReq getStartReq() const { lock_guard lg(m_mtx); return m_startReq; }
    SettingsReq getSettingsReq() const { lock_guard lg(m_mtx); return m_settingsReq; }
    ProgressReq getProgressReq() const { lock_guard lg(m_mtx); return m_progressReq; }
    ErrorReq getErrorReq() const { lock_guard lg(m_mtx); return m_errorReq; }
  // clang-format on

 private:
  mutable Status m_status;
  CommissionReq m_commissionReq;
  CommissionRes m_commissionRes;
  StartReq m_startReq;
  StartRes m_startRes;
  SettingsReq m_settingsReq;
  SettingsRes m_settingsRes;
  ProgressReq m_progressReq;
  ProgressRes m_progressRes;
  ErrorReq m_errorReq;
};

extern ThreadSharedData sd;

void thread();

time_t getReqInterval(time_t min, time_t max);

}  // namespace api::thread

#endif  // IG_API_API_H

``````



## vacuul-types
`clone_url`: https://github.com/vacuul-dev/vacuul-types.git


### LICENSE
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-types/LICENSE
`relative_path`: LICENSE
`format`: Arbitrary Binary Data
`size`: 1067   


``````
MIT License

Copyright (c) 2024 vacuul-dev

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

``````


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-types/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 14   


``````
# vacuul-types
``````


### src/models.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-types/src/models.ts
`relative_path`: src/models.ts
`format`: Arbitrary Binary Data
`size`: 1320   


``````
export interface Machine {
  id: string;
  createdAt: string;
  updatedAt: string;

  latitude: number;
  longitude: number;
  state: "active" | "inactive";

  location: string | null;
}

export interface Appointment {
  id: string;
  createdAt: string;
  updatedAt: string;

  userId: string;
  machineId: string;
  timeSlotId: string;
  therapyId: string;
  paymentId: string | null;
  state: "pending" | "acquired" | "completed" | "cancelled";
}

export interface Therapy {
  id: string;
  createdAt: string;
  updatedAt: string;

  appointmentId: string;
  settings: Record<string, any>;
  active: boolean;
}

export interface Payment {
  id: string;
  createdAt: string;
  updatedAt: string;

  userId: string;

  amountInCents: number;
  paymentState: "pending" | "completed" | "failed";
  stripePaymentMethodId: string;
}

export interface MachineError {
  id: string;
  createdAt: string;
  updatedAt: string;

  machineId: string;
  errorDetails: string;
  loggedAt: string;
}

export interface TimeSlot {
  id: string;
  createdAt: string;
  updatedAt: string;

  machineId: string;
  startTime: string;
  endTime: string;
  state: "reserved" | "assigned";
}

export interface Feedback {
  id: string;
  createdAt: string;
  updatedAt: string;

  appointmentId: string;
  rating: number;
  feedback: string;
}

``````



## test-deno-function
`clone_url`: https://github.com/vacuul-dev/test-deno-function.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/test-deno-function/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 986   


``````
# ⚡ Deno Starter Function

A simple starter function. Edit `src/main.ts` to get started and create something awesome! 🚀

## 🧰 Usage

### GET /ping

- Returns a "Pong" message.

**Response**

Sample `200` Response:

```text
Pong
```

### GET, POST, PUT, PATCH, DELETE /

- Returns a "Learn More" JSON response.

**Response**

Sample `200` Response:

```json
{
  "motto": "Build like a team of hundreds_",
  "learn": "https://appwrite.io/docs",
  "connect": "https://appwrite.io/discord",
  "getInspired": "https://builtwith.appwrite.io"
}
```

## ⚙️ Configuration

| Setting           | Value                    |
| ----------------- | ------------------------ |
| Runtime           | Deno (1.35)              |
| Entrypoint        | `src/main.ts`            |
| Build Commands    | `deno cache src/main.ts` |
| Permissions       | `any`                    |
| Timeout (Seconds) | 15                       |

## 🔒 Environment Variables

No environment variables required.

``````


### src/main.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/test-deno-function/src/main.ts
`relative_path`: src/main.ts
`format`: Arbitrary Binary Data
`size`: 1284   


``````
import { Client, Users } from "https://deno.land/x/appwrite@7.0.0/mod.ts";

// This Appwrite function will be executed every time your function is triggered
export default async ({ req, res, log, error }: any) => {
  // You can use the Appwrite SDK to interact with other services
  // For this example, we're using the Users service
  const client = new Client()
    .setEndpoint(Deno.env.get("APPWRITE_FUNCTION_API_ENDPOINT") ?? '')
    .setProject(Deno.env.get("APPWRITE_FUNCTION_PROJECT_ID") ?? '')
    .setKey(req.headers['x-appwrite-key'] ?? '');
  const users = new Users(client);

  try {
    const response = await users.list();
    // Log messages and errors to the Appwrite Console
    // These logs won't be seen by your end users
    log(`Total users: ${response.total}`);
  } catch(err) {
    error("Could not list users: " + err.message);
  }

  // The req object contains the request data
  if (req.path === "/ping") {
    // Use res object to respond with text(), json(), or binary()
    // Don't forget to return a response!
    return res.text("Pong");
  }

  return res.json({
    motto: "Build like a team of hundreds_",
    learn: "https://appwrite.io/docs",
    connect: "https://appwrite.io/discord",
    getInspired: "https://builtwith.appwrite.io",
  });
};

``````



## ListAppointments_v1
`clone_url`: https://github.com/vacuul-dev/ListAppointments_v1.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/ListAppointments_v1/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 986   


``````
# ⚡ Deno Starter Function

A simple starter function. Edit `src/main.ts` to get started and create something awesome! 🚀

## 🧰 Usage

### GET /ping

- Returns a "Pong" message.

**Response**

Sample `200` Response:

```text
Pong
```

### GET, POST, PUT, PATCH, DELETE /

- Returns a "Learn More" JSON response.

**Response**

Sample `200` Response:

```json
{
  "motto": "Build like a team of hundreds_",
  "learn": "https://appwrite.io/docs",
  "connect": "https://appwrite.io/discord",
  "getInspired": "https://builtwith.appwrite.io"
}
```

## ⚙️ Configuration

| Setting           | Value                    |
| ----------------- | ------------------------ |
| Runtime           | Deno (1.35)              |
| Entrypoint        | `src/main.ts`            |
| Build Commands    | `deno cache src/main.ts` |
| Permissions       | `any`                    |
| Timeout (Seconds) | 15                       |

## 🔒 Environment Variables

No environment variables required.

``````


### src/main.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/ListAppointments_v1/src/main.ts
`relative_path`: src/main.ts
`format`: Arbitrary Binary Data
`size`: 3405   


``````
import {
  Client,
  Databases,
  Models,
  Query,
} from "https://deno.land/x/appwrite@12.1.0/mod.ts";

type Appointment = {
  id: string;
  userId: string;
  machineId: string;
  startTime: Date;
  endTime: Date;
  createdAt: Date;
  updatedAt: Date;
  machineRelationId: string | null;
  treatmentId: string | null;
  paymentId: string | null;
  machineSlotId: string | null;
};

type AppointmentDocument = Models.Document & {
  userId: string;
  machineId: string;
  startTime: string;
  endTime: string;
  machineRelationId?: string;
  treatmentId?: string;
  paymentId?: string;
  machineSlotId?: string;
};

function mapDocumentToAppointment(doc: AppointmentDocument): Appointment {
  return {
    id: doc.$id,
    userId: doc.userId,
    machineId: doc.machineId,
    startTime: new Date(doc.startTime),
    endTime: new Date(doc.endTime),
    createdAt: new Date(doc.$createdAt),
    updatedAt: new Date(doc.$updatedAt),
    machineRelationId: doc.machineRelationId || null,
    treatmentId: doc.treatmentId || null,
    paymentId: doc.paymentId || null,
    machineSlotId: doc.machineSlotId || null,
  };
}

async function validateSessionWithJWT(client: Client, jwt: string): Promise<string> {
  try {
    client.setJWT(jwt);
    const user = await client.call("get", "/account");
    return user.$id;
  } catch (error) {
    throw new Error("Error validating JWT: " + (error as Error).message);
  }
}

const ListAppointments_v1 = async (
  client: Client,
  databaseId: string,
  appointmentsCollectionId: string,
  userId?: string
): Promise<Appointment[]> => {
  const databases = new Databases(client);

  const queries = userId ? [Query.equal("userId", userId)] : [];

  return await databases
    .listDocuments<AppointmentDocument>(databaseId, appointmentsCollectionId, queries)
    .then((response) => {
      if (!response.documents || !Array.isArray(response.documents)) {
        throw new Error("Response does not contain a valid documents array.");
      }
      return response.documents.map(mapDocumentToAppointment);
    });
};

export default async ({ req, res, log, error }: any) => {
  const client = new Client()
    .setEndpoint(Deno.env.get("APPWRITE_FUNCTION_API_ENDPOINT") ?? "")
    .setProject(Deno.env.get("APPWRITE_FUNCTION_PROJECT_ID") ?? "");

  const jwt = req.headers["x-appwrite-user-jwt"];
  if (!jwt) {
    error("Missing JWT in headers.");
    return res.json({ error: "Authentication required." }, 401);
  }

  const userId = await validateSessionWithJWT(client, jwt);
  if (!userId) {
    error("Invalid JWT or user session.");
    return res.json({ error: "Invalid session." }, 401);
  }

  const databaseId = Deno.env.get("APPWRITE_DATABASE_ID") ?? "";
  const appointmentsCollectionId =
    Deno.env.get("APPWRITE_APPOINTMENTS_COLLECTION_ID") ?? "";

  if (!databaseId || !appointmentsCollectionId) {
    error("Missing database ID or collection ID in environment variables.");
    return res.json({ error: "Internal server error. Please contact support." }, 500);
  }

  try {
    const appointments = await ListAppointments_v1(
      client,
      databaseId,
      appointmentsCollectionId,
      userId
    );

    log("Appointments retrieved successfully.");
    return res.json(appointments);
  } catch (err) {
    error(`Error retrieving appointments: ${err.message}`);
    return res.json({ error: "Unable to fetch appointments." }, 500);
  }
};

``````



## FindMachines_v1
`clone_url`: https://github.com/vacuul-dev/FindMachines_v1.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/FindMachines_v1/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 2865   


``````
# ⚡ FindMachines Function

This function, when invoked via an HTTP POST request, returns a list of machine objects from your connected data source. Each machine object includes details such as its `id`, `location`, `status`, and timestamps for creation and updates.

## 🧰 Usage

### POST /

Invoke this endpoint to retrieve a list of machines.

**Request Headers**

- `Content-Type: application/json`
- `X-Appwrite-Project: <YOUR_PROJECT_ID>`
- `X-Appwrite-Key: <YOUR_PROJECT_API_KEY>`

**Response**

A `200` response returns a JSON array of machine objects. Each object contains the following fields:

- `id`: Unique identifier of the machine.
- `location`: A string describing the machine’s location.
- `status`: The current status of the machine (e.g., `"active"`).
- `createdAt`: The timestamp (ISO 8601 format) of when the machine was created.
- `updatedAt`: The timestamp (ISO 8601 format) of when the machine was last updated.

**Sample `200` Response:**

```json
[
  {
    "id": 67522,
    "location": "Warehouse A\n",
    "status": "active",
    "createdAt": "2024-12-05T22:35:05.093Z",
    "updatedAt": "2024-12-05T22:35:05.093Z"
  }
]
```

## 🚀 Invoking the Function from a Flutter App

You can also invoke this function using the [Appwrite Flutter SDK](https://appwrite.io/docs/getting-started-for-flutter), enabling you to integrate the data retrieval into your mobile application seamlessly. Below is a sample snippet demonstrating how to create an execution of this function directly from your Flutter code:

```dart
import 'package:appwrite/appwrite.dart';

void main() {
  // Initialize the Appwrite client
  Client client = Client();

  // Set the Project ID and Endpoint
  client
    .setProject('<PROJECT_ID>'); // Replace with your project ID

  Functions functions = Functions(client);

  // Invoke the FindMachines function
  Future result = functions.createExecution(
    functionId: '<FUNCTION_ID>', // Replace with your function ID
    body: '{}', // No body is required for this function
    xasync: false,
    path: '/',
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Appwrite-Project': '<YOUR_PROJECT_ID>',
      'X-Appwrite-Key': '<YOUR_PROJECT_API_KEY>',
    },
  );

  result.then((response) {
    print(response); // JSON list of machines
  }).catchError((error) {
    print(error.response); // Error response
  });
}
```

## ⚙️ Configuration

| Setting           | Value                    |
| ----------------- | ------------------------ |
| Runtime           | Deno (1.35)              |
| Entrypoint        | `src/main.ts`            |
| Build Commands    | `deno cache src/main.ts` |
| Permissions       | `any`                    |
| Timeout (Seconds) | 15                       |

## 🔒 Environment Variables

No environment variables are required to run this function.

``````


### src/main.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/FindMachines_v1/src/main.ts
`relative_path`: src/main.ts
`format`: Arbitrary Binary Data
`size`: 3617   


``````
import {
  Client,
  Databases,
  Models,
} from "https://deno.land/x/appwrite@12.1.0/mod.ts";
import { Machine } from "https://raw.githubusercontent.com/vacuul-dev/vacuul-types/main/src/models.ts";

type MachineDocument = Models.Document & {
  location: string;
  latitude: number;
  longitude: number;
  status: "active" | "inactive";
};

type Filters = {
  date?: string;
  startTime?: string;
  endTime?: string;
  location?: string;
};

function buildQueries(filters?: Filters): string[] {
  const queries = [
    filters?.location ? `equal("location", ["${filters.location}"])` : "",
    filters?.date && filters?.startTime
      ? `greaterThanEqual("createdAt", "${filters.date}T${filters.startTime}")`
      : "",
    filters?.date && filters?.endTime
      ? `lessThanEqual("createdAt", "${filters.date}T${filters.endTime}")`
      : "",
  ].filter(Boolean);

  console.log("Generated queries:", queries);
  return queries;
}

function mapDocumentToMachine(doc: MachineDocument): Machine {
  console.log(`Mapping document ID: ${doc.$id}`);
  return {
    id: doc.$id,
    location: doc.location.replace(/\n/g, "").replace(/\\n/g, ""),
    latitude: doc.latitude,
    longitude: doc.longitude,
    state: doc.status,
    createdAt: doc.$createdAt,
    updatedAt: doc.$updatedAt,
  };
}

const ListMachines_v1 = async (
  client: Client,
  databaseId: string,
  machinesCollectionId: string,
  filters?: Filters
): Promise<Machine[]> => {
  const databases = new Databases(client);
  const queries = buildQueries(filters);

  return await databases
    .listDocuments<MachineDocument>(databaseId, machinesCollectionId, queries)
    .then((response) => {
      console.log("Raw documents retrieved:", JSON.stringify(response.documents));
      if (!response.documents || !Array.isArray(response.documents)) {
        return Promise.reject(
          new Error("Response does not contain a valid documents array.")
        );
      }

      const ids = response.documents.map((doc) => doc.$id);
      const duplicates = ids.filter((id, index) => ids.indexOf(id) !== index);

      if (duplicates.length > 0) {
        console.log("Duplicate IDs found:", duplicates);
      } else {
        console.log("No duplicate IDs found.");
      }

      return response.documents.map(mapDocumentToMachine);
    });
};

const euclideanDistance = (x1: number, y1: number, x2: number, y2: number) => {
  return Math.sqrt((x1 - x2) ** 2 + (y1 - y2) ** 2);
};

export default async ({ req, res, log, error }: any) => {
  const client = new Client()
    .setEndpoint(Deno.env.get("APPWRITE_FUNCTION_API_ENDPOINT") ?? "")
    .setProject(Deno.env.get("APPWRITE_FUNCTION_PROJECT_ID") ?? "")
    .setKey(req.headers["x-appwrite-key"] ?? "");

  const databaseId = Deno.env.get("APPWRITE_DATABASE_ID") ?? "";
  const machinesCollectionId =
    Deno.env.get("APPWRITE_MACHINES_COLLECTION_ID") ?? "";

  const default_lat = 52.520008;
  const default_lng = 13.404954;

  const input = req.bodyJson || {};

  log("Input: " + JSON.stringify(input));

  const lat = input?.lat || default_lat;
  const lng = input?.lng || default_lng;

  const machines = await ListMachines_v1(
    client,
    databaseId,
    machinesCollectionId
  );

  log("Machines retrieved successfully:", JSON.stringify(machines));

  const sorted_machines = machines.sort((a, b) => {
    const distA = euclideanDistance(a.latitude, a.longitude, lat, lng);
    const distB = euclideanDistance(b.latitude, b.longitude, lat, lng);

    return distA - distB;
  });

  log("Sorted machines:", JSON.stringify(sorted_machines));

  return res.json(sorted_machines);
};

``````



## AvailableTimeSlots_v1
`clone_url`: https://github.com/vacuul-dev/AvailableTimeSlots_v1.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/AvailableTimeSlots_v1/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 986   


``````
# ⚡ Deno Starter Function

A simple starter function. Edit `src/main.ts` to get started and create something awesome! 🚀

## 🧰 Usage

### GET /ping

- Returns a "Pong" message.

**Response**

Sample `200` Response:

```text
Pong
```

### GET, POST, PUT, PATCH, DELETE /

- Returns a "Learn More" JSON response.

**Response**

Sample `200` Response:

```json
{
  "motto": "Build like a team of hundreds_",
  "learn": "https://appwrite.io/docs",
  "connect": "https://appwrite.io/discord",
  "getInspired": "https://builtwith.appwrite.io"
}
```

## ⚙️ Configuration

| Setting           | Value                    |
| ----------------- | ------------------------ |
| Runtime           | Deno (1.35)              |
| Entrypoint        | `src/main.ts`            |
| Build Commands    | `deno cache src/main.ts` |
| Permissions       | `any`                    |
| Timeout (Seconds) | 15                       |

## 🔒 Environment Variables

No environment variables required.

``````


### src/main.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/AvailableTimeSlots_v1/src/main.ts
`relative_path`: src/main.ts
`format`: Arbitrary Binary Data
`size`: 2844   


``````
import {
  Client,
  Databases,
  Query,
} from "https://deno.land/x/appwrite@12.1.0/mod.ts";

async function fetchRegisteredTimeSlots(
  client: Client,
  databaseId: string,
  collectionId: string,
  machineId: string,
  startTime: string,
  endTime: string
): Promise<string[]> {
  const databases = new Databases(client);

  const queries: string[] = [
    Query.equal("machineId", [machineId]),
    Query.greaterThanEqual("startTime", startTime),
    Query.lessThanEqual("startTime", endTime),
  ];

  const response = await databases.listDocuments<{ startTime: string }>(
    databaseId,
    collectionId,
    queries
  );

  return response.documents.map((doc) => doc.startTime);
}

export default async ({ req, res, log, error }: any) => {
  log("Received raw request body: ", req.body);

  let body;
  try {
    body = typeof req.body === "string" ? JSON.parse(req.body) : req.body;
  } catch (err) {
    console.error("Invalid JSON body:", err.message);
    return res.json({ error: "Invalid JSON format" }, 400);
  }

  const { machineId, startTime, endTime, duration } = body || {};

  if (!machineId || !startTime || !endTime) {
    log("Missing required fields:", {
      machineId,
      startTime,
      endTime,
    });
    return res.json(
      { error: "machineId, startTime, and endTime are required" },
      400
    );
  }

  // Crear y configurar la instancia de Client
  const client = new Client()
    .setEndpoint(Deno.env.get("APPWRITE_FUNCTION_API_ENDPOINT") ?? "")
    .setProject(Deno.env.get("APPWRITE_FUNCTION_PROJECT_ID") ?? "")
    .setKey(req.headers["x-appwrite-key"] ?? "");

  const databaseId = Deno.env.get("APPWRITE_DATABASE_ID") ?? "";
  const collectionId = Deno.env.get("APPWRITE_TIMESLOTS_COLLECTION_ID") ?? "";

  try {
    log("Machine ID received: ", machineId);
    log("Generating slots from: ", startTime, " to ", endTime);

    const possibleSlots = generateTimeSlots(startTime, endTime, duration);

    const registeredSlots = await fetchRegisteredTimeSlots(
      client,
      databaseId,
      collectionId,
      machineId,
      startTime,
      endTime
    );

    const availableSlots = possibleSlots.filter(
      (slot) => !registeredSlots.includes(slot)
    );

    return res.json({
      possibleSlots,
      registeredSlots,
      availableSlots,
    });
  } catch (err) {
    console.error("Could not fetch available time slots: ", err.message);
    return res.json({ error: err.message }, 500);
  }
};

function generateTimeSlots(
  startTime: string,
  endTime: string,
  duration: number = 30
): string[] {
  const slots: string[] = [];
  let currentTime = new Date(startTime);
  const end = new Date(endTime);

  while (currentTime < end) {
    slots.push(currentTime.toISOString());
    currentTime.setMinutes(currentTime.getMinutes() + duration);
  }

  return slots;
}

``````



## StartBooking_v1
`clone_url`: https://github.com/vacuul-dev/StartBooking_v1.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/StartBooking_v1/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 986   


``````
# ⚡ Deno Starter Function

A simple starter function. Edit `src/main.ts` to get started and create something awesome! 🚀

## 🧰 Usage

### GET /ping

- Returns a "Pong" message.

**Response**

Sample `200` Response:

```text
Pong
```

### GET, POST, PUT, PATCH, DELETE /

- Returns a "Learn More" JSON response.

**Response**

Sample `200` Response:

```json
{
  "motto": "Build like a team of hundreds_",
  "learn": "https://appwrite.io/docs",
  "connect": "https://appwrite.io/discord",
  "getInspired": "https://builtwith.appwrite.io"
}
```

## ⚙️ Configuration

| Setting           | Value                    |
| ----------------- | ------------------------ |
| Runtime           | Deno (1.35)              |
| Entrypoint        | `src/main.ts`            |
| Build Commands    | `deno cache src/main.ts` |
| Permissions       | `any`                    |
| Timeout (Seconds) | 15                       |

## 🔒 Environment Variables

No environment variables required.

``````


### src/main.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/StartBooking_v1/src/main.ts
`relative_path`: src/main.ts
`format`: Arbitrary Binary Data
`size`: 8029   





## DoneBooking_v1
`clone_url`: https://github.com/vacuul-dev/DoneBooking_v1.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/DoneBooking_v1/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 986   


``````
# ⚡ Deno Starter Function

A simple starter function. Edit `src/main.ts` to get started and create something awesome! 🚀

## 🧰 Usage

### GET /ping

- Returns a "Pong" message.

**Response**

Sample `200` Response:

```text
Pong
```

### GET, POST, PUT, PATCH, DELETE /

- Returns a "Learn More" JSON response.

**Response**

Sample `200` Response:

```json
{
  "motto": "Build like a team of hundreds_",
  "learn": "https://appwrite.io/docs",
  "connect": "https://appwrite.io/discord",
  "getInspired": "https://builtwith.appwrite.io"
}
```

## ⚙️ Configuration

| Setting           | Value                    |
| ----------------- | ------------------------ |
| Runtime           | Deno (1.35)              |
| Entrypoint        | `src/main.ts`            |
| Build Commands    | `deno cache src/main.ts` |
| Permissions       | `any`                    |
| Timeout (Seconds) | 15                       |

## 🔒 Environment Variables

No environment variables required.

``````


### src/main.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/DoneBooking_v1/src/main.ts
`relative_path`: src/main.ts
`format`: Arbitrary Binary Data
`size`: 8752   





## ConfigureTherapy_v1
`clone_url`: https://github.com/vacuul-dev/ConfigureTherapy_v1.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/ConfigureTherapy_v1/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 986   


``````
# ⚡ Deno Starter Function

A simple starter function. Edit `src/main.ts` to get started and create something awesome! 🚀

## 🧰 Usage

### GET /ping

- Returns a "Pong" message.

**Response**

Sample `200` Response:

```text
Pong
```

### GET, POST, PUT, PATCH, DELETE /

- Returns a "Learn More" JSON response.

**Response**

Sample `200` Response:

```json
{
  "motto": "Build like a team of hundreds_",
  "learn": "https://appwrite.io/docs",
  "connect": "https://appwrite.io/discord",
  "getInspired": "https://builtwith.appwrite.io"
}
```

## ⚙️ Configuration

| Setting           | Value                    |
| ----------------- | ------------------------ |
| Runtime           | Deno (1.35)              |
| Entrypoint        | `src/main.ts`            |
| Build Commands    | `deno cache src/main.ts` |
| Permissions       | `any`                    |
| Timeout (Seconds) | 15                       |

## 🔒 Environment Variables

No environment variables required.

``````


### src/main.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/ConfigureTherapy_v1/src/main.ts
`relative_path`: src/main.ts
`format`: Arbitrary Binary Data
`size`: 5271   





## StartTherapy_v1
`clone_url`: https://github.com/vacuul-dev/StartTherapy_v1.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/StartTherapy_v1/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 986   


``````
# ⚡ Deno Starter Function

A simple starter function. Edit `src/main.ts` to get started and create something awesome! 🚀

## 🧰 Usage

### GET /ping

- Returns a "Pong" message.

**Response**

Sample `200` Response:

```text
Pong
```

### GET, POST, PUT, PATCH, DELETE /

- Returns a "Learn More" JSON response.

**Response**

Sample `200` Response:

```json
{
  "motto": "Build like a team of hundreds_",
  "learn": "https://appwrite.io/docs",
  "connect": "https://appwrite.io/discord",
  "getInspired": "https://builtwith.appwrite.io"
}
```

## ⚙️ Configuration

| Setting           | Value                    |
| ----------------- | ------------------------ |
| Runtime           | Deno (1.35)              |
| Entrypoint        | `src/main.ts`            |
| Build Commands    | `deno cache src/main.ts` |
| Permissions       | `any`                    |
| Timeout (Seconds) | 15                       |

## 🔒 Environment Variables

No environment variables required.

``````


### src/main.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/StartTherapy_v1/src/main.ts
`relative_path`: src/main.ts
`format`: Arbitrary Binary Data
`size`: 5749   





## SendFeedback_v1
`clone_url`: https://github.com/vacuul-dev/SendFeedback_v1.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/SendFeedback_v1/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 986   


``````
# ⚡ Deno Starter Function

A simple starter function. Edit `src/main.ts` to get started and create something awesome! 🚀

## 🧰 Usage

### GET /ping

- Returns a "Pong" message.

**Response**

Sample `200` Response:

```text
Pong
```

### GET, POST, PUT, PATCH, DELETE /

- Returns a "Learn More" JSON response.

**Response**

Sample `200` Response:

```json
{
  "motto": "Build like a team of hundreds_",
  "learn": "https://appwrite.io/docs",
  "connect": "https://appwrite.io/discord",
  "getInspired": "https://builtwith.appwrite.io"
}
```

## ⚙️ Configuration

| Setting           | Value                    |
| ----------------- | ------------------------ |
| Runtime           | Deno (1.35)              |
| Entrypoint        | `src/main.ts`            |
| Build Commands    | `deno cache src/main.ts` |
| Permissions       | `any`                    |
| Timeout (Seconds) | 15                       |

## 🔒 Environment Variables

No environment variables required.

``````


### src/main.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/SendFeedback_v1/src/main.ts
`relative_path`: src/main.ts
`format`: Arbitrary Binary Data
`size`: 3073   


``````
import { Client, Databases, ID, Account } from "https://deno.land/x/appwrite@12.1.0/mod.ts";

async function submitFeedback(
  client: Client,
  databaseId: string,
  feedbackCollectionId: string,
  appointmentId: string,
  feedbackMessage: string,
  feedbackRating: string,
  userId: string
): Promise<object> {
  const databases = new Databases(client);

  if (!appointmentId || !feedbackMessage || !["1", "2", "3", "4", "5"].includes(feedbackRating)) {
    throw new Error(
      "Invalid input. Ensure appointmentId, feedbackMessage, and feedbackRating (1-5 as strings) are provided."
    );
  }

  const feedbackId = ID.unique();

  const feedbackData = {
    feedbackId, 
    appointmentId,
    userId,
    feedbackMessage,
    feedbackRating,
    createdAt: new Date().toISOString(),
  };

  try {
    const response = await databases.createDocument(
      databaseId,
      feedbackCollectionId,
      feedbackId, 
      feedbackData
    );
    return response;
  } catch (err) {
    throw new Error("Failed to submit feedback: " + err.message);
  }
}

async function getUserId(client: Client, jwtToken: string): Promise<string> {
  const account = new Account(client);

  try {
    client.setJWT(jwtToken);
    const user = await account.get();
    return user.$id;
  } catch (err) {
    throw new Error("Failed to verify JWT. Invalid or expired token.");
  }
}

export default async ({ req, res, log, error }: any) => {
  log("[Handler] Request received for submitFeedback...");

  const client = new Client()
    .setEndpoint(Deno.env.get("APPWRITE_FUNCTION_API_ENDPOINT") ?? "")
    .setProject(Deno.env.get("APPWRITE_FUNCTION_PROJECT_ID") ?? "");

  const databaseId = Deno.env.get("APPWRITE_DATABASE_ID") ?? "";
  const feedbackCollectionId = Deno.env.get("APPWRITE_FEEDBACK_COLLECTION_ID") ?? "";

  const jwt = req.headers["x-appwrite-user-jwt"];
  if (!jwt) {
    log("[Handler Debug] JWT not found in headers.");
    return res.json({ error: "Authentication required. JWT is missing." }, 401);
  }

  let body;
  try {
    body = req.body ? JSON.parse(req.body) : req.bodyJson || {};
    log("[Handler Debug] Extracted body:", body);
  } catch (err) {
    return res.json({ error: "Invalid JSON format." }, 400);
  }

  const { appointmentId, feedbackMessage, feedbackRating } = body || {};
  if (!appointmentId || !feedbackMessage || feedbackRating === undefined) {
    return res.json(
      { error: "appointmentId, feedbackMessage, and feedbackRating are required." },
      400
    );
  }

  try {
    const userId = await getUserId(client, jwt);
    log("[Handler Debug] User ID:", userId);

    const feedback = await submitFeedback(
      client,
      databaseId,
      feedbackCollectionId,
      appointmentId,
      feedbackMessage,
      feedbackRating.toString(), 
      userId
    );

    return res.json({
      success: true,
      message: "Feedback submitted successfully.",
      feedback,
    });
  } catch (err) {
    error("[Handler] Error during execution:", err.message);
    return res.json({ error: err.message }, 500);
  }
};


``````



## vacuul_app_playground
`clone_url`: https://github.com/vacuul-dev/vacuul_app_playground.git


### macos/Runner.xcworkspace/contents.xcworkspacedata
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner.xcworkspace/contents.xcworkspacedata
`relative_path`: macos/Runner.xcworkspace/contents.xcworkspacedata
`format`: Extensible Markup Language
`size`: 152   




### macos/Runner.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`relative_path`: macos/Runner.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`format`: Extensible Markup Language
`size`: 238   




### macos/RunnerTests/RunnerTests.swift
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/RunnerTests/RunnerTests.swift
`relative_path`: macos/RunnerTests/RunnerTests.swift
`format`: Arbitrary Binary Data
`size`: 290   


``````
import Cocoa
import FlutterMacOS
import XCTest

class RunnerTests: XCTestCase {

  func testExample() {
    // If you add code to the Runner application, consider adding tests here.
    // See https://developer.apple.com/documentation/xctest for more information about using XCTest.
  }

}

``````


### macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_16.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_16.png
`relative_path`: macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_16.png
`format`: Portable Network Graphics
`size`: 520   




### macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_1024.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_1024.png
`relative_path`: macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_1024.png
`format`: Portable Network Graphics
`size`: 102994   




### macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_256.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_256.png
`relative_path`: macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_256.png
`format`: Portable Network Graphics
`size`: 14142   




### macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_64.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_64.png
`relative_path`: macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_64.png
`format`: Portable Network Graphics
`size`: 2218   




### macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_512.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_512.png
`relative_path`: macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_512.png
`format`: Portable Network Graphics
`size`: 36406   




### macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_128.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_128.png
`relative_path`: macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_128.png
`format`: Portable Network Graphics
`size`: 5680   




### macos/Runner/Assets.xcassets/AppIcon.appiconset/Contents.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Assets.xcassets/AppIcon.appiconset/Contents.json
`relative_path`: macos/Runner/Assets.xcassets/AppIcon.appiconset/Contents.json
`format`: Arbitrary Binary Data
`size`: 1291   


``````
{
  "images" : [
    {
      "size" : "16x16",
      "idiom" : "mac",
      "filename" : "app_icon_16.png",
      "scale" : "1x"
    },
    {
      "size" : "16x16",
      "idiom" : "mac",
      "filename" : "app_icon_32.png",
      "scale" : "2x"
    },
    {
      "size" : "32x32",
      "idiom" : "mac",
      "filename" : "app_icon_32.png",
      "scale" : "1x"
    },
    {
      "size" : "32x32",
      "idiom" : "mac",
      "filename" : "app_icon_64.png",
      "scale" : "2x"
    },
    {
      "size" : "128x128",
      "idiom" : "mac",
      "filename" : "app_icon_128.png",
      "scale" : "1x"
    },
    {
      "size" : "128x128",
      "idiom" : "mac",
      "filename" : "app_icon_256.png",
      "scale" : "2x"
    },
    {
      "size" : "256x256",
      "idiom" : "mac",
      "filename" : "app_icon_256.png",
      "scale" : "1x"
    },
    {
      "size" : "256x256",
      "idiom" : "mac",
      "filename" : "app_icon_512.png",
      "scale" : "2x"
    },
    {
      "size" : "512x512",
      "idiom" : "mac",
      "filename" : "app_icon_512.png",
      "scale" : "1x"
    },
    {
      "size" : "512x512",
      "idiom" : "mac",
      "filename" : "app_icon_1024.png",
      "scale" : "2x"
    }
  ],
  "info" : {
    "version" : 1,
    "author" : "xcode"
  }
}

``````


### macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_32.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_32.png
`relative_path`: macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_32.png
`format`: Portable Network Graphics
`size`: 1066   




### macos/Runner/DebugProfile.entitlements
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/DebugProfile.entitlements
`relative_path`: macos/Runner/DebugProfile.entitlements
`format`: Extensible Markup Language
`size`: 348   




### macos/Runner/Base.lproj/MainMenu.xib
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Base.lproj/MainMenu.xib
`relative_path`: macos/Runner/Base.lproj/MainMenu.xib
`format`: Extensible Markup Language
`size`: 23729   




### macos/Runner/MainFlutterWindow.swift
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/MainFlutterWindow.swift
`relative_path`: macos/Runner/MainFlutterWindow.swift
`format`: Arbitrary Binary Data
`size`: 388   


``````
import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow {
  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    let windowFrame = self.frame
    self.contentViewController = flutterViewController
    self.setFrame(windowFrame, display: true)

    RegisterGeneratedPlugins(registry: flutterViewController)

    super.awakeFromNib()
  }
}

``````


### macos/Runner/Configs/AppInfo.xcconfig
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Configs/AppInfo.xcconfig
`relative_path`: macos/Runner/Configs/AppInfo.xcconfig
`format`: Arbitrary Binary Data
`size`: 627   


``````
// Application-level settings for the Runner target.
//
// This may be replaced with something auto-generated from metadata (e.g., pubspec.yaml) in the
// future. If not, the values below would default to using the project name when this becomes a
// 'flutter create' template.

// The application's name. By default this is also the title of the Flutter window.
PRODUCT_NAME = vacuul_app_playground

// The application's bundle identifier
PRODUCT_BUNDLE_IDENTIFIER = com.example.vacuulAppPlayground

// The copyright displayed in application information
PRODUCT_COPYRIGHT = Copyright © 2024 com.example. All rights reserved.

``````


### macos/Runner/Configs/Debug.xcconfig
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Configs/Debug.xcconfig
`relative_path`: macos/Runner/Configs/Debug.xcconfig
`format`: Arbitrary Binary Data
`size`: 77   


``````
#include "../../Flutter/Flutter-Debug.xcconfig"
#include "Warnings.xcconfig"

``````


### macos/Runner/Configs/Release.xcconfig
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Configs/Release.xcconfig
`relative_path`: macos/Runner/Configs/Release.xcconfig
`format`: Arbitrary Binary Data
`size`: 79   


``````
#include "../../Flutter/Flutter-Release.xcconfig"
#include "Warnings.xcconfig"

``````


### macos/Runner/Configs/Warnings.xcconfig
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Configs/Warnings.xcconfig
`relative_path`: macos/Runner/Configs/Warnings.xcconfig
`format`: Arbitrary Binary Data
`size`: 580   


``````
WARNING_CFLAGS = -Wall -Wconditional-uninitialized -Wnullable-to-nonnull-conversion -Wmissing-method-return-type -Woverlength-strings
GCC_WARN_UNDECLARED_SELECTOR = YES
CLANG_UNDEFINED_BEHAVIOR_SANITIZER_NULLABILITY = YES
CLANG_WARN_UNGUARDED_AVAILABILITY = YES_AGGRESSIVE
CLANG_WARN__DUPLICATE_METHOD_MATCH = YES
CLANG_WARN_PRAGMA_PACK = YES
CLANG_WARN_STRICT_PROTOTYPES = YES
CLANG_WARN_COMMA = YES
GCC_WARN_STRICT_SELECTOR_MATCH = YES
CLANG_WARN_OBJC_REPEATED_USE_OF_WEAK = YES
CLANG_WARN_OBJC_IMPLICIT_RETAIN_SELF = YES
GCC_WARN_SHADOW = YES
CLANG_WARN_UNREACHABLE_CODE = YES

``````


### macos/Runner/AppDelegate.swift
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/AppDelegate.swift
`relative_path`: macos/Runner/AppDelegate.swift
`format`: Arbitrary Binary Data
`size`: 311   


``````
import Cocoa
import FlutterMacOS

@main
class AppDelegate: FlutterAppDelegate {
  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    return true
  }

  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }
}

``````


### macos/Runner/Info.plist
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Info.plist
`relative_path`: macos/Runner/Info.plist
`format`: Extensible Markup Language
`size`: 1060   




### macos/Runner/Release.entitlements
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner/Release.entitlements
`relative_path`: macos/Runner/Release.entitlements
`format`: Extensible Markup Language
`size`: 240   




### macos/Runner.xcodeproj/project.pbxproj
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner.xcodeproj/project.pbxproj
`relative_path`: macos/Runner.xcodeproj/project.pbxproj
`format`: Arbitrary Binary Data
`size`: 26522   




### macos/Runner.xcodeproj/project.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner.xcodeproj/project.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`relative_path`: macos/Runner.xcodeproj/project.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`format`: Extensible Markup Language
`size`: 238   




### macos/Runner.xcodeproj/xcshareddata/xcschemes/Runner.xcscheme
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Runner.xcodeproj/xcshareddata/xcschemes/Runner.xcscheme
`relative_path`: macos/Runner.xcodeproj/xcshareddata/xcschemes/Runner.xcscheme
`format`: Extensible Markup Language
`size`: 3707   




### macos/Flutter/Flutter-Debug.xcconfig
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Flutter/Flutter-Debug.xcconfig
`relative_path`: macos/Flutter/Flutter-Debug.xcconfig
`format`: Arbitrary Binary Data
`size`: 125   


``````
#include? "Pods/Target Support Files/Pods-Runner/Pods-Runner.debug.xcconfig"
#include "ephemeral/Flutter-Generated.xcconfig"

``````


### macos/Flutter/GeneratedPluginRegistrant.swift
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Flutter/GeneratedPluginRegistrant.swift
`relative_path`: macos/Flutter/GeneratedPluginRegistrant.swift
`format`: Arbitrary Binary Data
`size`: 869   


``````
//
//  Generated file. Do not edit.
//

import FlutterMacOS
import Foundation

import device_info_plus
import flutter_web_auth_2
import package_info_plus
import path_provider_foundation
import url_launcher_macos
import window_to_front

func RegisterGeneratedPlugins(registry: FlutterPluginRegistry) {
  DeviceInfoPlusMacosPlugin.register(with: registry.registrar(forPlugin: "DeviceInfoPlusMacosPlugin"))
  FlutterWebAuth2Plugin.register(with: registry.registrar(forPlugin: "FlutterWebAuth2Plugin"))
  FPPPackageInfoPlusPlugin.register(with: registry.registrar(forPlugin: "FPPPackageInfoPlusPlugin"))
  PathProviderPlugin.register(with: registry.registrar(forPlugin: "PathProviderPlugin"))
  UrlLauncherPlugin.register(with: registry.registrar(forPlugin: "UrlLauncherPlugin"))
  WindowToFrontPlugin.register(with: registry.registrar(forPlugin: "WindowToFrontPlugin"))
}

``````


### macos/Flutter/Flutter-Release.xcconfig
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Flutter/Flutter-Release.xcconfig
`relative_path`: macos/Flutter/Flutter-Release.xcconfig
`format`: Arbitrary Binary Data
`size`: 127   


``````
#include? "Pods/Target Support Files/Pods-Runner/Pods-Runner.release.xcconfig"
#include "ephemeral/Flutter-Generated.xcconfig"

``````


### macos/Podfile
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/macos/Podfile
`relative_path`: macos/Podfile
`format`: Arbitrary Binary Data
`size`: 1389   


``````
platform :osx, '10.14'

# CocoaPods analytics sends network stats synchronously affecting flutter build latency.
ENV['COCOAPODS_DISABLE_STATS'] = 'true'

project 'Runner', {
  'Debug' => :debug,
  'Profile' => :release,
  'Release' => :release,
}

def flutter_root
  generated_xcode_build_settings_path = File.expand_path(File.join('..', 'Flutter', 'ephemeral', 'Flutter-Generated.xcconfig'), __FILE__)
  unless File.exist?(generated_xcode_build_settings_path)
    raise "#{generated_xcode_build_settings_path} must exist. If you're running pod install manually, make sure \"flutter pub get\" is executed first"
  end

  File.foreach(generated_xcode_build_settings_path) do |line|
    matches = line.match(/FLUTTER_ROOT\=(.*)/)
    return matches[1].strip if matches
  end
  raise "FLUTTER_ROOT not found in #{generated_xcode_build_settings_path}. Try deleting Flutter-Generated.xcconfig, then run \"flutter pub get\""
end

require File.expand_path(File.join('packages', 'flutter_tools', 'bin', 'podhelper'), flutter_root)

flutter_macos_podfile_setup

target 'Runner' do
  use_frameworks!
  use_modular_headers!

  flutter_install_all_macos_pods File.dirname(File.realpath(__FILE__))
  target 'RunnerTests' do
    inherit! :search_paths
  end
end

post_install do |installer|
  installer.pods_project.targets.each do |target|
    flutter_additional_macos_build_settings(target)
  end
end

``````


### test/widget_test.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/test/widget_test.dart
`relative_path`: test/widget_test.dart
`format`: Arbitrary Binary Data
`size`: 1072   


``````
// This is a basic Flutter widget test.
//
// To perform an interaction with a widget in your test, use the WidgetTester
// utility in the flutter_test package. For example, you can send tap and scroll
// gestures. You can also use WidgetTester to find child widgets in the widget
// tree, read text, and verify that the values of widget properties are correct.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:vacuul_app_playground/main.dart';

void main() {
  testWidgets('Counter increments smoke test', (WidgetTester tester) async {
    // Build our app and trigger a frame.
    await tester.pumpWidget(const MyApp());

    // Verify that our counter starts at 0.
    expect(find.text('0'), findsOneWidget);
    expect(find.text('1'), findsNothing);

    // Tap the '+' icon and trigger a frame.
    await tester.tap(find.byIcon(Icons.add));
    await tester.pump();

    // Verify that our counter has incremented.
    expect(find.text('0'), findsNothing);
    expect(find.text('1'), findsOneWidget);
  });
}

``````


### web/index.html
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/web/index.html
`relative_path`: web/index.html
`format`: HyperText Markup Language
`size`: 1240   




### web/favicon.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/web/favicon.png
`relative_path`: web/favicon.png
`format`: Portable Network Graphics
`size`: 917   




### web/icons/Icon-192.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/web/icons/Icon-192.png
`relative_path`: web/icons/Icon-192.png
`format`: Portable Network Graphics
`size`: 5292   




### web/icons/Icon-maskable-192.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/web/icons/Icon-maskable-192.png
`relative_path`: web/icons/Icon-maskable-192.png
`format`: Portable Network Graphics
`size`: 5594   




### web/icons/Icon-maskable-512.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/web/icons/Icon-maskable-512.png
`relative_path`: web/icons/Icon-maskable-512.png
`format`: Portable Network Graphics
`size`: 20998   




### web/icons/Icon-512.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/web/icons/Icon-512.png
`relative_path`: web/icons/Icon-512.png
`format`: Portable Network Graphics
`size`: 8252   




### web/manifest.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/web/manifest.json
`relative_path`: web/manifest.json
`format`: Arbitrary Binary Data
`size`: 938   


``````
{
    "name": "vacuul_app_playground",
    "short_name": "vacuul_app_playground",
    "start_url": ".",
    "display": "standalone",
    "background_color": "#0175C2",
    "theme_color": "#0175C2",
    "description": "A new Flutter project.",
    "orientation": "portrait-primary",
    "prefer_related_applications": false,
    "icons": [
        {
            "src": "icons/Icon-192.png",
            "sizes": "192x192",
            "type": "image/png"
        },
        {
            "src": "icons/Icon-512.png",
            "sizes": "512x512",
            "type": "image/png"
        },
        {
            "src": "icons/Icon-maskable-192.png",
            "sizes": "192x192",
            "type": "image/png",
            "purpose": "maskable"
        },
        {
            "src": "icons/Icon-maskable-512.png",
            "sizes": "512x512",
            "type": "image/png",
            "purpose": "maskable"
        }
    ]
}

``````


### pubspec.lock
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/pubspec.lock
`relative_path`: pubspec.lock
`format`: Arbitrary Binary Data
`size`: 17358   




### ios/Runner.xcworkspace/contents.xcworkspacedata
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner.xcworkspace/contents.xcworkspacedata
`relative_path`: ios/Runner.xcworkspace/contents.xcworkspacedata
`format`: Extensible Markup Language
`size`: 224   




### ios/Runner.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`relative_path`: ios/Runner.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`format`: Extensible Markup Language
`size`: 238   




### ios/Runner.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`relative_path`: ios/Runner.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`format`: Extensible Markup Language
`size`: 226   




### ios/RunnerTests/RunnerTests.swift
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/RunnerTests/RunnerTests.swift
`relative_path`: ios/RunnerTests/RunnerTests.swift
`format`: Arbitrary Binary Data
`size`: 285   


``````
import Flutter
import UIKit
import XCTest

class RunnerTests: XCTestCase {

  func testExample() {
    // If you add code to the Runner application, consider adding tests here.
    // See https://developer.apple.com/documentation/xctest for more information about using XCTest.
  }

}

``````


### ios/Runner/Runner-Bridging-Header.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Runner-Bridging-Header.h
`relative_path`: ios/Runner/Runner-Bridging-Header.h
`format`: Arbitrary Binary Data
`size`: 38   


``````
#import "GeneratedPluginRegistrant.h"

``````


### ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@2x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@2x.png
`relative_path`: ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@2x.png
`format`: Portable Network Graphics
`size`: 68   




### ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@3x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@3x.png
`relative_path`: ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@3x.png
`format`: Portable Network Graphics
`size`: 68   




### ios/Runner/Assets.xcassets/LaunchImage.imageset/README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/LaunchImage.imageset/README.md
`relative_path`: ios/Runner/Assets.xcassets/LaunchImage.imageset/README.md
`format`: Arbitrary Binary Data
`size`: 336   


``````
# Launch Screen Assets

You can customize the launch screen with your own desired assets by replacing the image files in this directory.

You can also do it by opening your Flutter project's Xcode project with `open ios/Runner.xcworkspace`, selecting `Runner/Assets.xcassets` in the Project Navigator and dropping in the desired images.
``````


### ios/Runner/Assets.xcassets/LaunchImage.imageset/Contents.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/LaunchImage.imageset/Contents.json
`relative_path`: ios/Runner/Assets.xcassets/LaunchImage.imageset/Contents.json
`format`: Arbitrary Binary Data
`size`: 391   


``````
{
  "images" : [
    {
      "idiom" : "universal",
      "filename" : "LaunchImage.png",
      "scale" : "1x"
    },
    {
      "idiom" : "universal",
      "filename" : "LaunchImage@2x.png",
      "scale" : "2x"
    },
    {
      "idiom" : "universal",
      "filename" : "LaunchImage@3x.png",
      "scale" : "3x"
    }
  ],
  "info" : {
    "version" : 1,
    "author" : "xcode"
  }
}

``````


### ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage.png
`relative_path`: ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage.png
`format`: Portable Network Graphics
`size`: 68   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@2x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@2x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@2x.png
`format`: Portable Network Graphics
`size`: 1226   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@1x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@1x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@1x.png
`format`: Portable Network Graphics
`size`: 282   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@1x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@1x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@1x.png
`format`: Portable Network Graphics
`size`: 406   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@1x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@1x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@1x.png
`format`: Portable Network Graphics
`size`: 295   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-1024x1024@1x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-1024x1024@1x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-1024x1024@1x.png
`format`: Portable Network Graphics
`size`: 10932   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-83.5x83.5@2x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-83.5x83.5@2x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-83.5x83.5@2x.png
`format`: Portable Network Graphics
`size`: 1418   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@3x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@3x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@3x.png
`format`: Portable Network Graphics
`size`: 450   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Contents.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Contents.json
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Contents.json
`format`: Arbitrary Binary Data
`size`: 2519   


``````
{
  "images" : [
    {
      "size" : "20x20",
      "idiom" : "iphone",
      "filename" : "Icon-App-20x20@2x.png",
      "scale" : "2x"
    },
    {
      "size" : "20x20",
      "idiom" : "iphone",
      "filename" : "Icon-App-20x20@3x.png",
      "scale" : "3x"
    },
    {
      "size" : "29x29",
      "idiom" : "iphone",
      "filename" : "Icon-App-29x29@1x.png",
      "scale" : "1x"
    },
    {
      "size" : "29x29",
      "idiom" : "iphone",
      "filename" : "Icon-App-29x29@2x.png",
      "scale" : "2x"
    },
    {
      "size" : "29x29",
      "idiom" : "iphone",
      "filename" : "Icon-App-29x29@3x.png",
      "scale" : "3x"
    },
    {
      "size" : "40x40",
      "idiom" : "iphone",
      "filename" : "Icon-App-40x40@2x.png",
      "scale" : "2x"
    },
    {
      "size" : "40x40",
      "idiom" : "iphone",
      "filename" : "Icon-App-40x40@3x.png",
      "scale" : "3x"
    },
    {
      "size" : "60x60",
      "idiom" : "iphone",
      "filename" : "Icon-App-60x60@2x.png",
      "scale" : "2x"
    },
    {
      "size" : "60x60",
      "idiom" : "iphone",
      "filename" : "Icon-App-60x60@3x.png",
      "scale" : "3x"
    },
    {
      "size" : "20x20",
      "idiom" : "ipad",
      "filename" : "Icon-App-20x20@1x.png",
      "scale" : "1x"
    },
    {
      "size" : "20x20",
      "idiom" : "ipad",
      "filename" : "Icon-App-20x20@2x.png",
      "scale" : "2x"
    },
    {
      "size" : "29x29",
      "idiom" : "ipad",
      "filename" : "Icon-App-29x29@1x.png",
      "scale" : "1x"
    },
    {
      "size" : "29x29",
      "idiom" : "ipad",
      "filename" : "Icon-App-29x29@2x.png",
      "scale" : "2x"
    },
    {
      "size" : "40x40",
      "idiom" : "ipad",
      "filename" : "Icon-App-40x40@1x.png",
      "scale" : "1x"
    },
    {
      "size" : "40x40",
      "idiom" : "ipad",
      "filename" : "Icon-App-40x40@2x.png",
      "scale" : "2x"
    },
    {
      "size" : "76x76",
      "idiom" : "ipad",
      "filename" : "Icon-App-76x76@1x.png",
      "scale" : "1x"
    },
    {
      "size" : "76x76",
      "idiom" : "ipad",
      "filename" : "Icon-App-76x76@2x.png",
      "scale" : "2x"
    },
    {
      "size" : "83.5x83.5",
      "idiom" : "ipad",
      "filename" : "Icon-App-83.5x83.5@2x.png",
      "scale" : "2x"
    },
    {
      "size" : "1024x1024",
      "idiom" : "ios-marketing",
      "filename" : "Icon-App-1024x1024@1x.png",
      "scale" : "1x"
    }
  ],
  "info" : {
    "version" : 1,
    "author" : "xcode"
  }
}

``````


### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@2x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@2x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@2x.png
`format`: Portable Network Graphics
`size`: 406   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@3x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@3x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@3x.png
`format`: Portable Network Graphics
`size`: 704   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@2x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@2x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@2x.png
`format`: Portable Network Graphics
`size`: 586   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@3x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@3x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@3x.png
`format`: Portable Network Graphics
`size`: 1674   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@2x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@2x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@2x.png
`format`: Portable Network Graphics
`size`: 862   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@1x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@1x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@1x.png
`format`: Portable Network Graphics
`size`: 762   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@3x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@3x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@3x.png
`format`: Portable Network Graphics
`size`: 862   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@2x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@2x.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@2x.png
`format`: Portable Network Graphics
`size`: 462   




### ios/Runner/Base.lproj/LaunchScreen.storyboard
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Base.lproj/LaunchScreen.storyboard
`relative_path`: ios/Runner/Base.lproj/LaunchScreen.storyboard
`format`: Extensible Markup Language
`size`: 2377   




### ios/Runner/Base.lproj/Main.storyboard
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Base.lproj/Main.storyboard
`relative_path`: ios/Runner/Base.lproj/Main.storyboard
`format`: Extensible Markup Language
`size`: 1809   




### ios/Runner/AppDelegate.swift
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/AppDelegate.swift
`relative_path`: ios/Runner/AppDelegate.swift
`format`: Arbitrary Binary Data
`size`: 391   


``````
import Flutter
import UIKit

@main
@objc class AppDelegate: FlutterAppDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    GeneratedPluginRegistrant.register(with: self)
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
}

``````


### ios/Runner/Info.plist
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner/Info.plist
`relative_path`: ios/Runner/Info.plist
`format`: Extensible Markup Language
`size`: 1846   




### ios/Runner.xcodeproj/project.pbxproj
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner.xcodeproj/project.pbxproj
`relative_path`: ios/Runner.xcodeproj/project.pbxproj
`format`: Arbitrary Binary Data
`size`: 30759   




### ios/Runner.xcodeproj/project.xcworkspace/contents.xcworkspacedata
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner.xcodeproj/project.xcworkspace/contents.xcworkspacedata
`relative_path`: ios/Runner.xcodeproj/project.xcworkspace/contents.xcworkspacedata
`format`: Extensible Markup Language
`size`: 135   




### ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`relative_path`: ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`format`: Extensible Markup Language
`size`: 238   




### ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`relative_path`: ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`format`: Extensible Markup Language
`size`: 226   




### ios/Runner.xcodeproj/xcshareddata/xcschemes/Runner.xcscheme
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Runner.xcodeproj/xcshareddata/xcschemes/Runner.xcscheme
`relative_path`: ios/Runner.xcodeproj/xcshareddata/xcschemes/Runner.xcscheme
`format`: Extensible Markup Language
`size`: 3647   




### ios/Flutter/Debug.xcconfig
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Flutter/Debug.xcconfig
`relative_path`: ios/Flutter/Debug.xcconfig
`format`: Arbitrary Binary Data
`size`: 107   


``````
#include? "Pods/Target Support Files/Pods-Runner/Pods-Runner.debug.xcconfig"
#include "Generated.xcconfig"

``````


### ios/Flutter/Release.xcconfig
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Flutter/Release.xcconfig
`relative_path`: ios/Flutter/Release.xcconfig
`format`: Arbitrary Binary Data
`size`: 109   


``````
#include? "Pods/Target Support Files/Pods-Runner/Pods-Runner.release.xcconfig"
#include "Generated.xcconfig"

``````


### ios/Flutter/AppFrameworkInfo.plist
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Flutter/AppFrameworkInfo.plist
`relative_path`: ios/Flutter/AppFrameworkInfo.plist
`format`: Extensible Markup Language
`size`: 774   




### ios/Podfile
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Podfile
`relative_path`: ios/Podfile
`format`: Arbitrary Binary Data
`size`: 1412   


``````
# Uncomment this line to define a global platform for your project
platform :ios, '13.0'

# CocoaPods analytics sends network stats synchronously affecting flutter build latency.
ENV['COCOAPODS_DISABLE_STATS'] = 'true'

project 'Runner', {
  'Debug' => :debug,
  'Profile' => :release,
  'Release' => :release,
}

def flutter_root
  generated_xcode_build_settings_path = File.expand_path(File.join('..', 'Flutter', 'Generated.xcconfig'), __FILE__)
  unless File.exist?(generated_xcode_build_settings_path)
    raise "#{generated_xcode_build_settings_path} must exist. If you're running pod install manually, make sure flutter pub get is executed first"
  end

  File.foreach(generated_xcode_build_settings_path) do |line|
    matches = line.match(/FLUTTER_ROOT\=(.*)/)
    return matches[1].strip if matches
  end
  raise "FLUTTER_ROOT not found in #{generated_xcode_build_settings_path}. Try deleting Generated.xcconfig, then run flutter pub get"
end

require File.expand_path(File.join('packages', 'flutter_tools', 'bin', 'podhelper'), flutter_root)

flutter_ios_podfile_setup

target 'Runner' do
  use_frameworks!
  use_modular_headers!

  flutter_install_all_ios_pods File.dirname(File.realpath(__FILE__))
  target 'RunnerTests' do
    inherit! :search_paths
  end
end

post_install do |installer|
  installer.pods_project.targets.each do |target|
    flutter_additional_ios_build_settings(target)
  end
end

``````


### ios/build/XCBuildData/89321e3aa0911dd3868b367216423ff5.xcbuilddata/manifest.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/build/XCBuildData/89321e3aa0911dd3868b367216423ff5.xcbuilddata/manifest.json
`relative_path`: ios/build/XCBuildData/89321e3aa0911dd3868b367216423ff5.xcbuilddata/manifest.json
`format`: Arbitrary Binary Data
`size`: 362   


``````
{"client":{"name":"basic","version":0,"file-system":"device-agnostic","perform-ownership-analysis":"no"},"targets":{"":["<all>"]},"commands":{"<all>":{"tool":"phony","inputs":["<WorkspaceHeaderMapVFSFilesWritten>"],"outputs":["<all>"]},"P0:::Gate WorkspaceHeaderMapVFSFilesWritten":{"tool":"phony","inputs":[],"outputs":["<WorkspaceHeaderMapVFSFilesWritten>"]}}}
``````


### ios/Podfile.lock
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/ios/Podfile.lock
`relative_path`: ios/Podfile.lock
`format`: Arbitrary Binary Data
`size`: 3570   


``````
PODS:
  - device_info_plus (0.0.1):
    - Flutter
  - Flutter (1.0.0)
  - flutter_web_auth_2 (3.0.0):
    - Flutter
  - package_info_plus (0.4.5):
    - Flutter
  - path_provider_foundation (0.0.1):
    - Flutter
    - FlutterMacOS
  - Stripe (23.30.0):
    - StripeApplePay (= 23.30.0)
    - StripeCore (= 23.30.0)
    - StripePayments (= 23.30.0)
    - StripePaymentsUI (= 23.30.0)
    - StripeUICore (= 23.30.0)
  - stripe_ios (0.0.1):
    - Flutter
    - Stripe (~> 23.30.0)
    - StripeApplePay (~> 23.30.0)
    - StripeFinancialConnections (~> 23.30.0)
    - StripePayments (~> 23.30.0)
    - StripePaymentSheet (~> 23.30.0)
    - StripePaymentsUI (~> 23.30.0)
  - StripeApplePay (23.30.0):
    - StripeCore (= 23.30.0)
  - StripeCore (23.30.0)
  - StripeFinancialConnections (23.30.0):
    - StripeCore (= 23.30.0)
    - StripeUICore (= 23.30.0)
  - StripePayments (23.30.0):
    - StripeCore (= 23.30.0)
    - StripePayments/Stripe3DS2 (= 23.30.0)
  - StripePayments/Stripe3DS2 (23.30.0):
    - StripeCore (= 23.30.0)
  - StripePaymentSheet (23.30.0):
    - StripeApplePay (= 23.30.0)
    - StripeCore (= 23.30.0)
    - StripePayments (= 23.30.0)
    - StripePaymentsUI (= 23.30.0)
  - StripePaymentsUI (23.30.0):
    - StripeCore (= 23.30.0)
    - StripePayments (= 23.30.0)
    - StripeUICore (= 23.30.0)
  - StripeUICore (23.30.0):
    - StripeCore (= 23.30.0)
  - url_launcher_ios (0.0.1):
    - Flutter

DEPENDENCIES:
  - device_info_plus (from `.symlinks/plugins/device_info_plus/ios`)
  - Flutter (from `Flutter`)
  - flutter_web_auth_2 (from `.symlinks/plugins/flutter_web_auth_2/ios`)
  - package_info_plus (from `.symlinks/plugins/package_info_plus/ios`)
  - path_provider_foundation (from `.symlinks/plugins/path_provider_foundation/darwin`)
  - stripe_ios (from `.symlinks/plugins/stripe_ios/ios`)
  - url_launcher_ios (from `.symlinks/plugins/url_launcher_ios/ios`)

SPEC REPOS:
  trunk:
    - Stripe
    - StripeApplePay
    - StripeCore
    - StripeFinancialConnections
    - StripePayments
    - StripePaymentSheet
    - StripePaymentsUI
    - StripeUICore

EXTERNAL SOURCES:
  device_info_plus:
    :path: ".symlinks/plugins/device_info_plus/ios"
  Flutter:
    :path: Flutter
  flutter_web_auth_2:
    :path: ".symlinks/plugins/flutter_web_auth_2/ios"
  package_info_plus:
    :path: ".symlinks/plugins/package_info_plus/ios"
  path_provider_foundation:
    :path: ".symlinks/plugins/path_provider_foundation/darwin"
  stripe_ios:
    :path: ".symlinks/plugins/stripe_ios/ios"
  url_launcher_ios:
    :path: ".symlinks/plugins/url_launcher_ios/ios"

SPEC CHECKSUMS:
  device_info_plus: 71ffc6ab7634ade6267c7a93088ed7e4f74e5896
  Flutter: e0871f40cf51350855a761d2e70bf5af5b9b5de7
  flutter_web_auth_2: 3464a7c16dc6480b6194fc89913bae6e82f28405
  package_info_plus: af8e2ca6888548050f16fa2f1938db7b5a5df499
  path_provider_foundation: 080d55be775b7414fd5a5ef3ac137b97b097e564
  Stripe: 9757efc154de1d9615cbea4836d590bc4034d3a4
  stripe_ios: ac48e0488f95ac7ddea9475fd30f3d739e0bae52
  StripeApplePay: ca33933601302742623762157d587b79b942d073
  StripeCore: 2af250a2366ff2bbf64d4243c5f9bbf2a98b2aaf
  StripeFinancialConnections: 3ab1ef6182ec44e71c29e9a2100b663f9713ac20
  StripePayments: 658a16bd34d20c8185aa281866227b9e1743300e
  StripePaymentSheet: eac031f76d7fbb4f52df9b9c39be5be671ca4c07
  StripePaymentsUI: 7d7cffb2ecfc0d6b5ac3a4488c02893a5ff6cc77
  StripeUICore: bb102d453b1e1a10a37f810bc0a9aa0675fb17fd
  url_launcher_ios: 694010445543906933d732453a59da0a173ae33d

PODFILE CHECKSUM: a57f30d18f102dd3ce366b1d62a55ecbef2158e5

COCOAPODS: 1.16.2

``````


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 564   


``````
# vacuul_app_playground

A new Flutter project.

## Getting Started

This project is a starting point for a Flutter application.

A few resources to get you started if this is your first Flutter project:

- [Lab: Write your first Flutter app](https://docs.flutter.dev/get-started/codelab)
- [Cookbook: Useful Flutter samples](https://docs.flutter.dev/cookbook)

For help getting started with Flutter development, view the
[online documentation](https://docs.flutter.dev/), which offers tutorials,
samples, guidance on mobile development, and a full API reference.

``````


### pubspec.yaml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/pubspec.yaml
`relative_path`: pubspec.yaml
`format`: Arbitrary Binary Data
`size`: 3923   


``````
name: vacuul_app_playground
description: "A new Flutter project."
# The following line prevents the package from being accidentally published to
# pub.dev using `flutter pub publish`. This is preferred for private packages.
publish_to: 'none' # Remove this line if you wish to publish to pub.dev

# The following defines the version and build number for your application.
# A version number is three numbers separated by dots, like 1.2.43
# followed by an optional build number separated by a +.
# Both the version and the builder number may be overridden in flutter
# build by specifying --build-name and --build-number, respectively.
# In Android, build-name is used as versionName while build-number used as versionCode.
# Read more about Android versioning at https://developer.android.com/studio/publish/versioning
# In iOS, build-name is used as CFBundleShortVersionString while build-number is used as CFBundleVersion.
# Read more about iOS versioning at
# https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html
# In Windows, build-name is used as the major, minor, and patch parts
# of the product and file versions while build-number is used as the build suffix.
version: 1.0.0+1

environment:
  sdk: ^3.6.0

# Dependencies specify other packages that your package needs in order to work.
# To automatically upgrade your package dependencies to the latest versions
# consider running `flutter pub upgrade --major-versions`. Alternatively,
# dependencies can be manually updated by changing the version numbers below to
# the latest version available on pub.dev. To see which dependencies have newer
# versions available, run `flutter pub outdated`.
dependencies:
  flutter:
    sdk: flutter

  # The following adds the Cupertino Icons font to your application.
  # Use with the CupertinoIcons class for iOS style icons.
  cupertino_icons: ^1.0.8
  appwrite: ^13.0.0
  flutter_stripe: ^11.3.0
  flutter_stripe_web: ^6.3.0

dev_dependencies:
  flutter_test:
    sdk: flutter

  # The "flutter_lints" package below contains a set of recommended lints to
  # encourage good coding practices. The lint set provided by the package is
  # activated in the `analysis_options.yaml` file located at the root of your
  # package. See that file for information about deactivating specific lint
  # rules and activating additional ones.
  flutter_lints: ^5.0.0

# For information on the generic Dart part of this file, see the
# following page: https://dart.dev/tools/pub/pubspec

# The following section is specific to Flutter packages.
flutter:

  # The following line ensures that the Material Icons font is
  # included with your application, so that you can use the icons in
  # the material Icons class.
  uses-material-design: true

  # To add assets to your application, add an assets section, like this:
  # assets:
  #   - images/a_dot_burr.jpeg
  #   - images/a_dot_ham.jpeg

  # An image asset can refer to one or more resolution-specific "variants", see
  # https://flutter.dev/to/resolution-aware-images

  # For details regarding adding assets from package dependencies, see
  # https://flutter.dev/to/asset-from-package

  # To add custom fonts to your application, add a fonts section here,
  # in this "flutter" section. Each entry in this list should have a
  # "family" key with the font family name, and a "fonts" key with a
  # list giving the asset and other descriptors for the font. For
  # example:
  # fonts:
  #   - family: Schyler
  #     fonts:
  #       - asset: fonts/Schyler-Regular.ttf
  #       - asset: fonts/Schyler-Italic.ttf
  #         style: italic
  #   - family: Trajan Pro
  #     fonts:
  #       - asset: fonts/TrajanPro.ttf
  #       - asset: fonts/TrajanPro_Bold.ttf
  #         weight: 700
  #
  # For details regarding fonts from package dependencies,
  # see https://flutter.dev/to/font-from-package

``````


### linux/CMakeLists.txt
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/linux/CMakeLists.txt
`relative_path`: linux/CMakeLists.txt
`format`: Arbitrary Binary Data
`size`: 4781   


``````
# Project-level configuration.
cmake_minimum_required(VERSION 3.13)
project(runner LANGUAGES CXX)

# The name of the executable created for the application. Change this to change
# the on-disk name of your application.
set(BINARY_NAME "vacuul_app_playground")
# The unique GTK application identifier for this application. See:
# https://wiki.gnome.org/HowDoI/ChooseApplicationID
set(APPLICATION_ID "com.example.vacuul_app_playground")

# Explicitly opt in to modern CMake behaviors to avoid warnings with recent
# versions of CMake.
cmake_policy(SET CMP0063 NEW)

# Load bundled libraries from the lib/ directory relative to the binary.
set(CMAKE_INSTALL_RPATH "$ORIGIN/lib")

# Root filesystem for cross-building.
if(FLUTTER_TARGET_PLATFORM_SYSROOT)
  set(CMAKE_SYSROOT ${FLUTTER_TARGET_PLATFORM_SYSROOT})
  set(CMAKE_FIND_ROOT_PATH ${CMAKE_SYSROOT})
  set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
  set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)
  set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
  set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
endif()

# Define build configuration options.
if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
  set(CMAKE_BUILD_TYPE "Debug" CACHE
    STRING "Flutter build mode" FORCE)
  set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS
    "Debug" "Profile" "Release")
endif()

# Compilation settings that should be applied to most targets.
#
# Be cautious about adding new options here, as plugins use this function by
# default. In most cases, you should add new options to specific targets instead
# of modifying this function.
function(APPLY_STANDARD_SETTINGS TARGET)
  target_compile_features(${TARGET} PUBLIC cxx_std_14)
  target_compile_options(${TARGET} PRIVATE -Wall -Werror)
  target_compile_options(${TARGET} PRIVATE "$<$<NOT:$<CONFIG:Debug>>:-O3>")
  target_compile_definitions(${TARGET} PRIVATE "$<$<NOT:$<CONFIG:Debug>>:NDEBUG>")
endfunction()

# Flutter library and tool build rules.
set(FLUTTER_MANAGED_DIR "${CMAKE_CURRENT_SOURCE_DIR}/flutter")
add_subdirectory(${FLUTTER_MANAGED_DIR})

# System-level dependencies.
find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK REQUIRED IMPORTED_TARGET gtk+-3.0)

# Application build; see runner/CMakeLists.txt.
add_subdirectory("runner")

# Run the Flutter tool portions of the build. This must not be removed.
add_dependencies(${BINARY_NAME} flutter_assemble)

# Only the install-generated bundle's copy of the executable will launch
# correctly, since the resources must in the right relative locations. To avoid
# people trying to run the unbundled copy, put it in a subdirectory instead of
# the default top-level location.
set_target_properties(${BINARY_NAME}
  PROPERTIES
  RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/intermediates_do_not_run"
)


# Generated plugin build rules, which manage building the plugins and adding
# them to the application.
include(flutter/generated_plugins.cmake)


# === Installation ===
# By default, "installing" just makes a relocatable bundle in the build
# directory.
set(BUILD_BUNDLE_DIR "${PROJECT_BINARY_DIR}/bundle")
if(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
  set(CMAKE_INSTALL_PREFIX "${BUILD_BUNDLE_DIR}" CACHE PATH "..." FORCE)
endif()

# Start with a clean build bundle directory every time.
install(CODE "
  file(REMOVE_RECURSE \"${BUILD_BUNDLE_DIR}/\")
  " COMPONENT Runtime)

set(INSTALL_BUNDLE_DATA_DIR "${CMAKE_INSTALL_PREFIX}/data")
set(INSTALL_BUNDLE_LIB_DIR "${CMAKE_INSTALL_PREFIX}/lib")

install(TARGETS ${BINARY_NAME} RUNTIME DESTINATION "${CMAKE_INSTALL_PREFIX}"
  COMPONENT Runtime)

install(FILES "${FLUTTER_ICU_DATA_FILE}" DESTINATION "${INSTALL_BUNDLE_DATA_DIR}"
  COMPONENT Runtime)

install(FILES "${FLUTTER_LIBRARY}" DESTINATION "${INSTALL_BUNDLE_LIB_DIR}"
  COMPONENT Runtime)

foreach(bundled_library ${PLUGIN_BUNDLED_LIBRARIES})
  install(FILES "${bundled_library}"
    DESTINATION "${INSTALL_BUNDLE_LIB_DIR}"
    COMPONENT Runtime)
endforeach(bundled_library)

# Copy the native assets provided by the build.dart from all packages.
set(NATIVE_ASSETS_DIR "${PROJECT_BUILD_DIR}native_assets/linux/")
install(DIRECTORY "${NATIVE_ASSETS_DIR}"
   DESTINATION "${INSTALL_BUNDLE_LIB_DIR}"
   COMPONENT Runtime)

# Fully re-copy the assets directory on each build to avoid having stale files
# from a previous install.
set(FLUTTER_ASSET_DIR_NAME "flutter_assets")
install(CODE "
  file(REMOVE_RECURSE \"${INSTALL_BUNDLE_DATA_DIR}/${FLUTTER_ASSET_DIR_NAME}\")
  " COMPONENT Runtime)
install(DIRECTORY "${PROJECT_BUILD_DIR}/${FLUTTER_ASSET_DIR_NAME}"
  DESTINATION "${INSTALL_BUNDLE_DATA_DIR}" COMPONENT Runtime)

# Install the AOT library on non-Debug builds only.
if(NOT CMAKE_BUILD_TYPE MATCHES "Debug")
  install(FILES "${AOT_LIBRARY}" DESTINATION "${INSTALL_BUNDLE_LIB_DIR}"
    COMPONENT Runtime)
endif()

``````


### linux/runner/main.cc
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/linux/runner/main.cc
`relative_path`: linux/runner/main.cc
`format`: Arbitrary Binary Data
`size`: 180   


``````
#include "my_application.h"

int main(int argc, char** argv) {
  g_autoptr(MyApplication) app = my_application_new();
  return g_application_run(G_APPLICATION(app), argc, argv);
}

``````


### linux/runner/CMakeLists.txt
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/linux/runner/CMakeLists.txt
`relative_path`: linux/runner/CMakeLists.txt
`format`: Arbitrary Binary Data
`size`: 974   


``````
cmake_minimum_required(VERSION 3.13)
project(runner LANGUAGES CXX)

# Define the application target. To change its name, change BINARY_NAME in the
# top-level CMakeLists.txt, not the value here, or `flutter run` will no longer
# work.
#
# Any new source files that you add to the application should be added here.
add_executable(${BINARY_NAME}
  "main.cc"
  "my_application.cc"
  "${FLUTTER_MANAGED_DIR}/generated_plugin_registrant.cc"
)

# Apply the standard set of build settings. This can be removed for applications
# that need different build settings.
apply_standard_settings(${BINARY_NAME})

# Add preprocessor definitions for the application ID.
add_definitions(-DAPPLICATION_ID="${APPLICATION_ID}")

# Add dependency libraries. Add any application-specific dependencies here.
target_link_libraries(${BINARY_NAME} PRIVATE flutter)
target_link_libraries(${BINARY_NAME} PRIVATE PkgConfig::GTK)

target_include_directories(${BINARY_NAME} PRIVATE "${CMAKE_SOURCE_DIR}")

``````


### linux/runner/my_application.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/linux/runner/my_application.h
`relative_path`: linux/runner/my_application.h
`format`: Arbitrary Binary Data
`size`: 388   


``````
#ifndef FLUTTER_MY_APPLICATION_H_
#define FLUTTER_MY_APPLICATION_H_

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(MyApplication, my_application, MY, APPLICATION,
                     GtkApplication)

/**
 * my_application_new:
 *
 * Creates a new Flutter-based application.
 *
 * Returns: a new #MyApplication.
 */
MyApplication* my_application_new();

#endif  // FLUTTER_MY_APPLICATION_H_

``````


### linux/runner/my_application.cc
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/linux/runner/my_application.cc
`relative_path`: linux/runner/my_application.cc
`format`: Arbitrary Binary Data
`size`: 4776   


``````
#include "my_application.h"

#include <flutter_linux/flutter_linux.h>
#ifdef GDK_WINDOWING_X11
#include <gdk/gdkx.h>
#endif

#include "flutter/generated_plugin_registrant.h"

struct _MyApplication {
  GtkApplication parent_instance;
  char** dart_entrypoint_arguments;
};

G_DEFINE_TYPE(MyApplication, my_application, GTK_TYPE_APPLICATION)

// Implements GApplication::activate.
static void my_application_activate(GApplication* application) {
  MyApplication* self = MY_APPLICATION(application);
  GtkWindow* window =
      GTK_WINDOW(gtk_application_window_new(GTK_APPLICATION(application)));

  // Use a header bar when running in GNOME as this is the common style used
  // by applications and is the setup most users will be using (e.g. Ubuntu
  // desktop).
  // If running on X and not using GNOME then just use a traditional title bar
  // in case the window manager does more exotic layout, e.g. tiling.
  // If running on Wayland assume the header bar will work (may need changing
  // if future cases occur).
  gboolean use_header_bar = TRUE;
#ifdef GDK_WINDOWING_X11
  GdkScreen* screen = gtk_window_get_screen(window);
  if (GDK_IS_X11_SCREEN(screen)) {
    const gchar* wm_name = gdk_x11_screen_get_window_manager_name(screen);
    if (g_strcmp0(wm_name, "GNOME Shell") != 0) {
      use_header_bar = FALSE;
    }
  }
#endif
  if (use_header_bar) {
    GtkHeaderBar* header_bar = GTK_HEADER_BAR(gtk_header_bar_new());
    gtk_widget_show(GTK_WIDGET(header_bar));
    gtk_header_bar_set_title(header_bar, "vacuul_app_playground");
    gtk_header_bar_set_show_close_button(header_bar, TRUE);
    gtk_window_set_titlebar(window, GTK_WIDGET(header_bar));
  } else {
    gtk_window_set_title(window, "vacuul_app_playground");
  }

  gtk_window_set_default_size(window, 1280, 720);
  gtk_widget_show(GTK_WIDGET(window));

  g_autoptr(FlDartProject) project = fl_dart_project_new();
  fl_dart_project_set_dart_entrypoint_arguments(project, self->dart_entrypoint_arguments);

  FlView* view = fl_view_new(project);
  gtk_widget_show(GTK_WIDGET(view));
  gtk_container_add(GTK_CONTAINER(window), GTK_WIDGET(view));

  fl_register_plugins(FL_PLUGIN_REGISTRY(view));

  gtk_widget_grab_focus(GTK_WIDGET(view));
}

// Implements GApplication::local_command_line.
static gboolean my_application_local_command_line(GApplication* application, gchar*** arguments, int* exit_status) {
  MyApplication* self = MY_APPLICATION(application);
  // Strip out the first argument as it is the binary name.
  self->dart_entrypoint_arguments = g_strdupv(*arguments + 1);

  g_autoptr(GError) error = nullptr;
  if (!g_application_register(application, nullptr, &error)) {
     g_warning("Failed to register: %s", error->message);
     *exit_status = 1;
     return TRUE;
  }

  g_application_activate(application);
  *exit_status = 0;

  return TRUE;
}

// Implements GApplication::startup.
static void my_application_startup(GApplication* application) {
  //MyApplication* self = MY_APPLICATION(object);

  // Perform any actions required at application startup.

  G_APPLICATION_CLASS(my_application_parent_class)->startup(application);
}

// Implements GApplication::shutdown.
static void my_application_shutdown(GApplication* application) {
  //MyApplication* self = MY_APPLICATION(object);

  // Perform any actions required at application shutdown.

  G_APPLICATION_CLASS(my_application_parent_class)->shutdown(application);
}

// Implements GObject::dispose.
static void my_application_dispose(GObject* object) {
  MyApplication* self = MY_APPLICATION(object);
  g_clear_pointer(&self->dart_entrypoint_arguments, g_strfreev);
  G_OBJECT_CLASS(my_application_parent_class)->dispose(object);
}

static void my_application_class_init(MyApplicationClass* klass) {
  G_APPLICATION_CLASS(klass)->activate = my_application_activate;
  G_APPLICATION_CLASS(klass)->local_command_line = my_application_local_command_line;
  G_APPLICATION_CLASS(klass)->startup = my_application_startup;
  G_APPLICATION_CLASS(klass)->shutdown = my_application_shutdown;
  G_OBJECT_CLASS(klass)->dispose = my_application_dispose;
}

static void my_application_init(MyApplication* self) {}

MyApplication* my_application_new() {
  // Set the program name to the application ID, which helps various systems
  // like GTK and desktop environments map this running application to its
  // corresponding .desktop file. This ensures better integration by allowing
  // the application to be recognized beyond its binary name.
  g_set_prgname(APPLICATION_ID);

  return MY_APPLICATION(g_object_new(my_application_get_type(),
                                     "application-id", APPLICATION_ID,
                                     "flags", G_APPLICATION_NON_UNIQUE,
                                     nullptr));
}

``````


### linux/flutter/generated_plugin_registrant.cc
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/linux/flutter/generated_plugin_registrant.cc
`relative_path`: linux/flutter/generated_plugin_registrant.cc
`format`: Arbitrary Binary Data
`size`: 706   


``````
//
//  Generated file. Do not edit.
//

// clang-format off

#include "generated_plugin_registrant.h"

#include <url_launcher_linux/url_launcher_plugin.h>
#include <window_to_front/window_to_front_plugin.h>

void fl_register_plugins(FlPluginRegistry* registry) {
  g_autoptr(FlPluginRegistrar) url_launcher_linux_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "UrlLauncherPlugin");
  url_launcher_plugin_register_with_registrar(url_launcher_linux_registrar);
  g_autoptr(FlPluginRegistrar) window_to_front_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "WindowToFrontPlugin");
  window_to_front_plugin_register_with_registrar(window_to_front_registrar);
}

``````


### linux/flutter/CMakeLists.txt
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/linux/flutter/CMakeLists.txt
`relative_path`: linux/flutter/CMakeLists.txt
`format`: Arbitrary Binary Data
`size`: 2815   


``````
# This file controls Flutter-level build steps. It should not be edited.
cmake_minimum_required(VERSION 3.10)

set(EPHEMERAL_DIR "${CMAKE_CURRENT_SOURCE_DIR}/ephemeral")

# Configuration provided via flutter tool.
include(${EPHEMERAL_DIR}/generated_config.cmake)

# TODO: Move the rest of this into files in ephemeral. See
# https://github.com/flutter/flutter/issues/57146.

# Serves the same purpose as list(TRANSFORM ... PREPEND ...),
# which isn't available in 3.10.
function(list_prepend LIST_NAME PREFIX)
    set(NEW_LIST "")
    foreach(element ${${LIST_NAME}})
        list(APPEND NEW_LIST "${PREFIX}${element}")
    endforeach(element)
    set(${LIST_NAME} "${NEW_LIST}" PARENT_SCOPE)
endfunction()

# === Flutter Library ===
# System-level dependencies.
find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK REQUIRED IMPORTED_TARGET gtk+-3.0)
pkg_check_modules(GLIB REQUIRED IMPORTED_TARGET glib-2.0)
pkg_check_modules(GIO REQUIRED IMPORTED_TARGET gio-2.0)

set(FLUTTER_LIBRARY "${EPHEMERAL_DIR}/libflutter_linux_gtk.so")

# Published to parent scope for install step.
set(FLUTTER_LIBRARY ${FLUTTER_LIBRARY} PARENT_SCOPE)
set(FLUTTER_ICU_DATA_FILE "${EPHEMERAL_DIR}/icudtl.dat" PARENT_SCOPE)
set(PROJECT_BUILD_DIR "${PROJECT_DIR}/build/" PARENT_SCOPE)
set(AOT_LIBRARY "${PROJECT_DIR}/build/lib/libapp.so" PARENT_SCOPE)

list(APPEND FLUTTER_LIBRARY_HEADERS
  "fl_basic_message_channel.h"
  "fl_binary_codec.h"
  "fl_binary_messenger.h"
  "fl_dart_project.h"
  "fl_engine.h"
  "fl_json_message_codec.h"
  "fl_json_method_codec.h"
  "fl_message_codec.h"
  "fl_method_call.h"
  "fl_method_channel.h"
  "fl_method_codec.h"
  "fl_method_response.h"
  "fl_plugin_registrar.h"
  "fl_plugin_registry.h"
  "fl_standard_message_codec.h"
  "fl_standard_method_codec.h"
  "fl_string_codec.h"
  "fl_value.h"
  "fl_view.h"
  "flutter_linux.h"
)
list_prepend(FLUTTER_LIBRARY_HEADERS "${EPHEMERAL_DIR}/flutter_linux/")
add_library(flutter INTERFACE)
target_include_directories(flutter INTERFACE
  "${EPHEMERAL_DIR}"
)
target_link_libraries(flutter INTERFACE "${FLUTTER_LIBRARY}")
target_link_libraries(flutter INTERFACE
  PkgConfig::GTK
  PkgConfig::GLIB
  PkgConfig::GIO
)
add_dependencies(flutter flutter_assemble)

# === Flutter tool backend ===
# _phony_ is a non-existent file to force this command to run every time,
# since currently there's no way to get a full input/output list from the
# flutter tool.
add_custom_command(
  OUTPUT ${FLUTTER_LIBRARY} ${FLUTTER_LIBRARY_HEADERS}
    ${CMAKE_CURRENT_BINARY_DIR}/_phony_
  COMMAND ${CMAKE_COMMAND} -E env
    ${FLUTTER_TOOL_ENVIRONMENT}
    "${FLUTTER_ROOT}/packages/flutter_tools/bin/tool_backend.sh"
      ${FLUTTER_TARGET_PLATFORM} ${CMAKE_BUILD_TYPE}
  VERBATIM
)
add_custom_target(flutter_assemble DEPENDS
  "${FLUTTER_LIBRARY}"
  ${FLUTTER_LIBRARY_HEADERS}
)

``````


### linux/flutter/generated_plugins.cmake
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/linux/flutter/generated_plugins.cmake
`relative_path`: linux/flutter/generated_plugins.cmake
`format`: Arbitrary Binary Data
`size`: 778   


``````
#
# Generated file, do not edit.
#

list(APPEND FLUTTER_PLUGIN_LIST
  url_launcher_linux
  window_to_front
)

list(APPEND FLUTTER_FFI_PLUGIN_LIST
)

set(PLUGIN_BUNDLED_LIBRARIES)

foreach(plugin ${FLUTTER_PLUGIN_LIST})
  add_subdirectory(flutter/ephemeral/.plugin_symlinks/${plugin}/linux plugins/${plugin})
  target_link_libraries(${BINARY_NAME} PRIVATE ${plugin}_plugin)
  list(APPEND PLUGIN_BUNDLED_LIBRARIES $<TARGET_FILE:${plugin}_plugin>)
  list(APPEND PLUGIN_BUNDLED_LIBRARIES ${${plugin}_bundled_libraries})
endforeach(plugin)

foreach(ffi_plugin ${FLUTTER_FFI_PLUGIN_LIST})
  add_subdirectory(flutter/ephemeral/.plugin_symlinks/${ffi_plugin}/linux plugins/${ffi_plugin})
  list(APPEND PLUGIN_BUNDLED_LIBRARIES ${${ffi_plugin}_bundled_libraries})
endforeach(ffi_plugin)

``````


### linux/flutter/generated_plugin_registrant.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/linux/flutter/generated_plugin_registrant.h
`relative_path`: linux/flutter/generated_plugin_registrant.h
`format`: Arbitrary Binary Data
`size`: 303   


``````
//
//  Generated file. Do not edit.
//

// clang-format off

#ifndef GENERATED_PLUGIN_REGISTRANT_
#define GENERATED_PLUGIN_REGISTRANT_

#include <flutter_linux/flutter_linux.h>

// Registers Flutter plugins.
void fl_register_plugins(FlPluginRegistry* registry);

#endif  // GENERATED_PLUGIN_REGISTRANT_

``````


### android/app/build.gradle
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/build.gradle
`relative_path`: android/app/build.gradle
`format`: Arbitrary Binary Data
`size`: 1390   


``````
plugins {
    id "com.android.application"
    id "kotlin-android"
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id "dev.flutter.flutter-gradle-plugin"
}

android {
    namespace = "com.example.vacuul_app_playground"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_1_8
    }

    defaultConfig {
        // TODO: Specify your own unique Application ID (https://developer.android.com/studio/build/application-id.html).
        applicationId = "com.example.vacuul_app_playground"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    buildTypes {
        release {
            // TODO: Add your own signing config for the release build.
            // Signing with the debug keys for now, so `flutter run --release` works.
            signingConfig = signingConfigs.debug
        }
    }
}

flutter {
    source = "../.."
}

``````


### android/app/src/profile/AndroidManifest.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/profile/AndroidManifest.xml
`relative_path`: android/app/src/profile/AndroidManifest.xml
`format`: Arbitrary Binary Data
`size`: 378   


``````
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <!-- The INTERNET permission is required for development. Specifically,
         the Flutter tool needs it to communicate with the running application
         to allow setting breakpoints, to provide hot reload, etc.
    -->
    <uses-permission android:name="android.permission.INTERNET"/>
</manifest>

``````


### android/app/src/main/res/mipmap-mdpi/ic_launcher.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/main/res/mipmap-mdpi/ic_launcher.png
`relative_path`: android/app/src/main/res/mipmap-mdpi/ic_launcher.png
`format`: Portable Network Graphics
`size`: 442   




### android/app/src/main/res/mipmap-hdpi/ic_launcher.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/main/res/mipmap-hdpi/ic_launcher.png
`relative_path`: android/app/src/main/res/mipmap-hdpi/ic_launcher.png
`format`: Portable Network Graphics
`size`: 544   




### android/app/src/main/res/drawable/launch_background.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/main/res/drawable/launch_background.xml
`relative_path`: android/app/src/main/res/drawable/launch_background.xml
`format`: Extensible Markup Language
`size`: 434   




### android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png
`relative_path`: android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png
`format`: Portable Network Graphics
`size`: 1443   




### android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png
`relative_path`: android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png
`format`: Portable Network Graphics
`size`: 1031   




### android/app/src/main/res/values-night/styles.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/main/res/values-night/styles.xml
`relative_path`: android/app/src/main/res/values-night/styles.xml
`format`: Extensible Markup Language
`size`: 995   




### android/app/src/main/res/values/styles.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/main/res/values/styles.xml
`relative_path`: android/app/src/main/res/values/styles.xml
`format`: Extensible Markup Language
`size`: 996   




### android/app/src/main/res/drawable-v21/launch_background.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/main/res/drawable-v21/launch_background.xml
`relative_path`: android/app/src/main/res/drawable-v21/launch_background.xml
`format`: Extensible Markup Language
`size`: 438   




### android/app/src/main/res/mipmap-xhdpi/ic_launcher.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/main/res/mipmap-xhdpi/ic_launcher.png
`relative_path`: android/app/src/main/res/mipmap-xhdpi/ic_launcher.png
`format`: Portable Network Graphics
`size`: 721   




### android/app/src/main/AndroidManifest.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/main/AndroidManifest.xml
`relative_path`: android/app/src/main/AndroidManifest.xml
`format`: Arbitrary Binary Data
`size`: 2209   


``````
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application
        android:label="vacuul_app_playground"
        android:name="${applicationName}"
        android:icon="@mipmap/ic_launcher">
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:launchMode="singleTop"
            android:taskAffinity=""
            android:theme="@style/LaunchTheme"
            android:configChanges="orientation|keyboardHidden|keyboard|screenSize|smallestScreenSize|locale|layoutDirection|fontScale|screenLayout|density|uiMode"
            android:hardwareAccelerated="true"
            android:windowSoftInputMode="adjustResize">
            <!-- Specifies an Android theme to apply to this Activity as soon as
                 the Android process has started. This theme is visible to the user
                 while the Flutter UI initializes. After that, this theme continues
                 to determine the Window background behind the Flutter UI. -->
            <meta-data
              android:name="io.flutter.embedding.android.NormalTheme"
              android:resource="@style/NormalTheme"
              />
            <intent-filter>
                <action android:name="android.intent.action.MAIN"/>
                <category android:name="android.intent.category.LAUNCHER"/>
            </intent-filter>
        </activity>
        <!-- Don't delete the meta-data below.
             This is used by the Flutter tool to generate GeneratedPluginRegistrant.java -->
        <meta-data
            android:name="flutterEmbedding"
            android:value="2" />
    </application>
    <!-- Required to query activities that can process text, see:
         https://developer.android.com/training/package-visibility and
         https://developer.android.com/reference/android/content/Intent#ACTION_PROCESS_TEXT.

         In particular, this is used by the Flutter engine in io.flutter.plugin.text.ProcessTextPlugin. -->
    <queries>
        <intent>
            <action android:name="android.intent.action.PROCESS_TEXT"/>
            <data android:mimeType="text/plain"/>
        </intent>
    </queries>
</manifest>

``````


### android/app/src/main/kotlin/com/example/vacuul_app_playground/MainActivity.kt
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/main/kotlin/com/example/vacuul_app_playground/MainActivity.kt
`relative_path`: android/app/src/main/kotlin/com/example/vacuul_app_playground/MainActivity.kt
`format`: Arbitrary Binary Data
`size`: 134   


``````
package com.example.vacuul_app_playground

import io.flutter.embedding.android.FlutterActivity

class MainActivity: FlutterActivity()

``````


### android/app/src/debug/AndroidManifest.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/app/src/debug/AndroidManifest.xml
`relative_path`: android/app/src/debug/AndroidManifest.xml
`format`: Arbitrary Binary Data
`size`: 378   


``````
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <!-- The INTERNET permission is required for development. Specifically,
         the Flutter tool needs it to communicate with the running application
         to allow setting breakpoints, to provide hot reload, etc.
    -->
    <uses-permission android:name="android.permission.INTERNET"/>
</manifest>

``````


### android/gradle/wrapper/gradle-wrapper.properties
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/gradle/wrapper/gradle-wrapper.properties
`relative_path`: android/gradle/wrapper/gradle-wrapper.properties
`format`: Arbitrary Binary Data
`size`: 200   


``````
distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-8.3-all.zip

``````


### android/build.gradle
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/build.gradle
`relative_path`: android/build.gradle
`format`: Arbitrary Binary Data
`size`: 322   


``````
allprojects {
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.buildDir = "../build"
subprojects {
    project.buildDir = "${rootProject.buildDir}/${project.name}"
}
subprojects {
    project.evaluationDependsOn(":app")
}

tasks.register("clean", Delete) {
    delete rootProject.buildDir
}

``````


### android/gradle.properties
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/gradle.properties
`relative_path`: android/gradle.properties
`format`: Arbitrary Binary Data
`size`: 135   


``````
org.gradle.jvmargs=-Xmx4G -XX:MaxMetaspaceSize=2G -XX:+HeapDumpOnOutOfMemoryError
android.useAndroidX=true
android.enableJetifier=true

``````


### android/settings.gradle
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/android/settings.gradle
`relative_path`: android/settings.gradle
`format`: Arbitrary Binary Data
`size`: 727   


``````
pluginManagement {
    def flutterSdkPath = {
        def properties = new Properties()
        file("local.properties").withInputStream { properties.load(it) }
        def flutterSdkPath = properties.getProperty("flutter.sdk")
        assert flutterSdkPath != null, "flutter.sdk not set in local.properties"
        return flutterSdkPath
    }()

    includeBuild("$flutterSdkPath/packages/flutter_tools/gradle")

    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

plugins {
    id "dev.flutter.flutter-plugin-loader" version "1.0.0"
    id "com.android.application" version "8.1.0" apply false
    id "org.jetbrains.kotlin.android" version "1.8.22" apply false
}

include ":app"

``````


### lib/main.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/lib/main.dart
`relative_path`: lib/main.dart
`format`: Arbitrary Binary Data
`size`: 6403   




### analysis_options.yaml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/analysis_options.yaml
`relative_path`: analysis_options.yaml
`format`: Arbitrary Binary Data
`size`: 1420   


``````
# This file configures the analyzer, which statically analyzes Dart code to
# check for errors, warnings, and lints.
#
# The issues identified by the analyzer are surfaced in the UI of Dart-enabled
# IDEs (https://dart.dev/tools#ides-and-editors). The analyzer can also be
# invoked from the command line by running `flutter analyze`.

# The following line activates a set of recommended lints for Flutter apps,
# packages, and plugins designed to encourage good coding practices.
include: package:flutter_lints/flutter.yaml

linter:
  # The lint rules applied to this project can be customized in the
  # section below to disable rules from the `package:flutter_lints/flutter.yaml`
  # included above or to enable additional rules. A list of all available lints
  # and their documentation is published at https://dart.dev/lints.
  #
  # Instead of disabling a lint rule for the entire project in the
  # section below, it can also be suppressed for a single line of code
  # or a specific dart file by using the `// ignore: name_of_lint` and
  # `// ignore_for_file: name_of_lint` syntax on the line or in the file
  # producing the lint.
  rules:
    # avoid_print: false  # Uncomment to disable the `avoid_print` rule
    # prefer_single_quotes: true  # Uncomment to enable the `prefer_single_quotes` rule

# Additional information about this file can be found at
# https://dart.dev/guides/language/analysis-options

``````


### windows/CMakeLists.txt
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/CMakeLists.txt
`relative_path`: windows/CMakeLists.txt
`format`: Arbitrary Binary Data
`size`: 4178   


``````
# Project-level configuration.
cmake_minimum_required(VERSION 3.14)
project(vacuul_app_playground LANGUAGES CXX)

# The name of the executable created for the application. Change this to change
# the on-disk name of your application.
set(BINARY_NAME "vacuul_app_playground")

# Explicitly opt in to modern CMake behaviors to avoid warnings with recent
# versions of CMake.
cmake_policy(VERSION 3.14...3.25)

# Define build configuration option.
get_property(IS_MULTICONFIG GLOBAL PROPERTY GENERATOR_IS_MULTI_CONFIG)
if(IS_MULTICONFIG)
  set(CMAKE_CONFIGURATION_TYPES "Debug;Profile;Release"
    CACHE STRING "" FORCE)
else()
  if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
    set(CMAKE_BUILD_TYPE "Debug" CACHE
      STRING "Flutter build mode" FORCE)
    set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS
      "Debug" "Profile" "Release")
  endif()
endif()
# Define settings for the Profile build mode.
set(CMAKE_EXE_LINKER_FLAGS_PROFILE "${CMAKE_EXE_LINKER_FLAGS_RELEASE}")
set(CMAKE_SHARED_LINKER_FLAGS_PROFILE "${CMAKE_SHARED_LINKER_FLAGS_RELEASE}")
set(CMAKE_C_FLAGS_PROFILE "${CMAKE_C_FLAGS_RELEASE}")
set(CMAKE_CXX_FLAGS_PROFILE "${CMAKE_CXX_FLAGS_RELEASE}")

# Use Unicode for all projects.
add_definitions(-DUNICODE -D_UNICODE)

# Compilation settings that should be applied to most targets.
#
# Be cautious about adding new options here, as plugins use this function by
# default. In most cases, you should add new options to specific targets instead
# of modifying this function.
function(APPLY_STANDARD_SETTINGS TARGET)
  target_compile_features(${TARGET} PUBLIC cxx_std_17)
  target_compile_options(${TARGET} PRIVATE /W4 /WX /wd"4100")
  target_compile_options(${TARGET} PRIVATE /EHsc)
  target_compile_definitions(${TARGET} PRIVATE "_HAS_EXCEPTIONS=0")
  target_compile_definitions(${TARGET} PRIVATE "$<$<CONFIG:Debug>:_DEBUG>")
endfunction()

# Flutter library and tool build rules.
set(FLUTTER_MANAGED_DIR "${CMAKE_CURRENT_SOURCE_DIR}/flutter")
add_subdirectory(${FLUTTER_MANAGED_DIR})

# Application build; see runner/CMakeLists.txt.
add_subdirectory("runner")


# Generated plugin build rules, which manage building the plugins and adding
# them to the application.
include(flutter/generated_plugins.cmake)


# === Installation ===
# Support files are copied into place next to the executable, so that it can
# run in place. This is done instead of making a separate bundle (as on Linux)
# so that building and running from within Visual Studio will work.
set(BUILD_BUNDLE_DIR "$<TARGET_FILE_DIR:${BINARY_NAME}>")
# Make the "install" step default, as it's required to run.
set(CMAKE_VS_INCLUDE_INSTALL_TO_DEFAULT_BUILD 1)
if(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
  set(CMAKE_INSTALL_PREFIX "${BUILD_BUNDLE_DIR}" CACHE PATH "..." FORCE)
endif()

set(INSTALL_BUNDLE_DATA_DIR "${CMAKE_INSTALL_PREFIX}/data")
set(INSTALL_BUNDLE_LIB_DIR "${CMAKE_INSTALL_PREFIX}")

install(TARGETS ${BINARY_NAME} RUNTIME DESTINATION "${CMAKE_INSTALL_PREFIX}"
  COMPONENT Runtime)

install(FILES "${FLUTTER_ICU_DATA_FILE}" DESTINATION "${INSTALL_BUNDLE_DATA_DIR}"
  COMPONENT Runtime)

install(FILES "${FLUTTER_LIBRARY}" DESTINATION "${INSTALL_BUNDLE_LIB_DIR}"
  COMPONENT Runtime)

if(PLUGIN_BUNDLED_LIBRARIES)
  install(FILES "${PLUGIN_BUNDLED_LIBRARIES}"
    DESTINATION "${INSTALL_BUNDLE_LIB_DIR}"
    COMPONENT Runtime)
endif()

# Copy the native assets provided by the build.dart from all packages.
set(NATIVE_ASSETS_DIR "${PROJECT_BUILD_DIR}native_assets/windows/")
install(DIRECTORY "${NATIVE_ASSETS_DIR}"
   DESTINATION "${INSTALL_BUNDLE_LIB_DIR}"
   COMPONENT Runtime)

# Fully re-copy the assets directory on each build to avoid having stale files
# from a previous install.
set(FLUTTER_ASSET_DIR_NAME "flutter_assets")
install(CODE "
  file(REMOVE_RECURSE \"${INSTALL_BUNDLE_DATA_DIR}/${FLUTTER_ASSET_DIR_NAME}\")
  " COMPONENT Runtime)
install(DIRECTORY "${PROJECT_BUILD_DIR}/${FLUTTER_ASSET_DIR_NAME}"
  DESTINATION "${INSTALL_BUNDLE_DATA_DIR}" COMPONENT Runtime)

# Install the AOT library on non-Debug builds only.
install(FILES "${AOT_LIBRARY}" DESTINATION "${INSTALL_BUNDLE_DATA_DIR}"
  CONFIGURATIONS Profile;Release
  COMPONENT Runtime)

``````


### windows/runner/flutter_window.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/flutter_window.cpp
`relative_path`: windows/runner/flutter_window.cpp
`format`: Arbitrary Binary Data
`size`: 2122   


``````
#include "flutter_window.h"

#include <optional>

#include "flutter/generated_plugin_registrant.h"

FlutterWindow::FlutterWindow(const flutter::DartProject& project)
    : project_(project) {}

FlutterWindow::~FlutterWindow() {}

bool FlutterWindow::OnCreate() {
  if (!Win32Window::OnCreate()) {
    return false;
  }

  RECT frame = GetClientArea();

  // The size here must match the window dimensions to avoid unnecessary surface
  // creation / destruction in the startup path.
  flutter_controller_ = std::make_unique<flutter::FlutterViewController>(
      frame.right - frame.left, frame.bottom - frame.top, project_);
  // Ensure that basic setup of the controller was successful.
  if (!flutter_controller_->engine() || !flutter_controller_->view()) {
    return false;
  }
  RegisterPlugins(flutter_controller_->engine());
  SetChildContent(flutter_controller_->view()->GetNativeWindow());

  flutter_controller_->engine()->SetNextFrameCallback([&]() {
    this->Show();
  });

  // Flutter can complete the first frame before the "show window" callback is
  // registered. The following call ensures a frame is pending to ensure the
  // window is shown. It is a no-op if the first frame hasn't completed yet.
  flutter_controller_->ForceRedraw();

  return true;
}

void FlutterWindow::OnDestroy() {
  if (flutter_controller_) {
    flutter_controller_ = nullptr;
  }

  Win32Window::OnDestroy();
}

LRESULT
FlutterWindow::MessageHandler(HWND hwnd, UINT const message,
                              WPARAM const wparam,
                              LPARAM const lparam) noexcept {
  // Give Flutter, including plugins, an opportunity to handle window messages.
  if (flutter_controller_) {
    std::optional<LRESULT> result =
        flutter_controller_->HandleTopLevelWindowProc(hwnd, message, wparam,
                                                      lparam);
    if (result) {
      return *result;
    }
  }

  switch (message) {
    case WM_FONTCHANGE:
      flutter_controller_->engine()->ReloadSystemFonts();
      break;
  }

  return Win32Window::MessageHandler(hwnd, message, wparam, lparam);
}

``````


### windows/runner/utils.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/utils.h
`relative_path`: windows/runner/utils.h
`format`: Arbitrary Binary Data
`size`: 672   


``````
#ifndef RUNNER_UTILS_H_
#define RUNNER_UTILS_H_

#include <string>
#include <vector>

// Creates a console for the process, and redirects stdout and stderr to
// it for both the runner and the Flutter library.
void CreateAndAttachConsole();

// Takes a null-terminated wchar_t* encoded in UTF-16 and returns a std::string
// encoded in UTF-8. Returns an empty std::string on failure.
std::string Utf8FromUtf16(const wchar_t* utf16_string);

// Gets the command line arguments passed in as a std::vector<std::string>,
// encoded in UTF-8. Returns an empty std::vector<std::string> on failure.
std::vector<std::string> GetCommandLineArguments();

#endif  // RUNNER_UTILS_H_

``````


### windows/runner/utils.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/utils.cpp
`relative_path`: windows/runner/utils.cpp
`format`: Arbitrary Binary Data
`size`: 1797   


``````
#include "utils.h"

#include <flutter_windows.h>
#include <io.h>
#include <stdio.h>
#include <windows.h>

#include <iostream>

void CreateAndAttachConsole() {
  if (::AllocConsole()) {
    FILE *unused;
    if (freopen_s(&unused, "CONOUT$", "w", stdout)) {
      _dup2(_fileno(stdout), 1);
    }
    if (freopen_s(&unused, "CONOUT$", "w", stderr)) {
      _dup2(_fileno(stdout), 2);
    }
    std::ios::sync_with_stdio();
    FlutterDesktopResyncOutputStreams();
  }
}

std::vector<std::string> GetCommandLineArguments() {
  // Convert the UTF-16 command line arguments to UTF-8 for the Engine to use.
  int argc;
  wchar_t** argv = ::CommandLineToArgvW(::GetCommandLineW(), &argc);
  if (argv == nullptr) {
    return std::vector<std::string>();
  }

  std::vector<std::string> command_line_arguments;

  // Skip the first argument as it's the binary name.
  for (int i = 1; i < argc; i++) {
    command_line_arguments.push_back(Utf8FromUtf16(argv[i]));
  }

  ::LocalFree(argv);

  return command_line_arguments;
}

std::string Utf8FromUtf16(const wchar_t* utf16_string) {
  if (utf16_string == nullptr) {
    return std::string();
  }
  unsigned int target_length = ::WideCharToMultiByte(
      CP_UTF8, WC_ERR_INVALID_CHARS, utf16_string,
      -1, nullptr, 0, nullptr, nullptr)
    -1; // remove the trailing null character
  int input_length = (int)wcslen(utf16_string);
  std::string utf8_string;
  if (target_length == 0 || target_length > utf8_string.max_size()) {
    return utf8_string;
  }
  utf8_string.resize(target_length);
  int converted_length = ::WideCharToMultiByte(
      CP_UTF8, WC_ERR_INVALID_CHARS, utf16_string,
      input_length, utf8_string.data(), target_length, nullptr, nullptr);
  if (converted_length == 0) {
    return std::string();
  }
  return utf8_string;
}

``````


### windows/runner/runner.exe.manifest
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/runner.exe.manifest
`relative_path`: windows/runner/runner.exe.manifest
`format`: Extensible Markup Language
`size`: 602   




### windows/runner/CMakeLists.txt
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/CMakeLists.txt
`relative_path`: windows/runner/CMakeLists.txt
`format`: Arbitrary Binary Data
`size`: 1796   


``````
cmake_minimum_required(VERSION 3.14)
project(runner LANGUAGES CXX)

# Define the application target. To change its name, change BINARY_NAME in the
# top-level CMakeLists.txt, not the value here, or `flutter run` will no longer
# work.
#
# Any new source files that you add to the application should be added here.
add_executable(${BINARY_NAME} WIN32
  "flutter_window.cpp"
  "main.cpp"
  "utils.cpp"
  "win32_window.cpp"
  "${FLUTTER_MANAGED_DIR}/generated_plugin_registrant.cc"
  "Runner.rc"
  "runner.exe.manifest"
)

# Apply the standard set of build settings. This can be removed for applications
# that need different build settings.
apply_standard_settings(${BINARY_NAME})

# Add preprocessor definitions for the build version.
target_compile_definitions(${BINARY_NAME} PRIVATE "FLUTTER_VERSION=\"${FLUTTER_VERSION}\"")
target_compile_definitions(${BINARY_NAME} PRIVATE "FLUTTER_VERSION_MAJOR=${FLUTTER_VERSION_MAJOR}")
target_compile_definitions(${BINARY_NAME} PRIVATE "FLUTTER_VERSION_MINOR=${FLUTTER_VERSION_MINOR}")
target_compile_definitions(${BINARY_NAME} PRIVATE "FLUTTER_VERSION_PATCH=${FLUTTER_VERSION_PATCH}")
target_compile_definitions(${BINARY_NAME} PRIVATE "FLUTTER_VERSION_BUILD=${FLUTTER_VERSION_BUILD}")

# Disable Windows macros that collide with C++ standard library functions.
target_compile_definitions(${BINARY_NAME} PRIVATE "NOMINMAX")

# Add dependency libraries and include directories. Add any application-specific
# dependencies here.
target_link_libraries(${BINARY_NAME} PRIVATE flutter flutter_wrapper_app)
target_link_libraries(${BINARY_NAME} PRIVATE "dwmapi.lib")
target_include_directories(${BINARY_NAME} PRIVATE "${CMAKE_SOURCE_DIR}")

# Run the Flutter tool portions of the build. This must not be removed.
add_dependencies(${BINARY_NAME} flutter_assemble)

``````


### windows/runner/win32_window.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/win32_window.h
`relative_path`: windows/runner/win32_window.h
`format`: Arbitrary Binary Data
`size`: 3522   


``````
#ifndef RUNNER_WIN32_WINDOW_H_
#define RUNNER_WIN32_WINDOW_H_

#include <windows.h>

#include <functional>
#include <memory>
#include <string>

// A class abstraction for a high DPI-aware Win32 Window. Intended to be
// inherited from by classes that wish to specialize with custom
// rendering and input handling
class Win32Window {
 public:
  struct Point {
    unsigned int x;
    unsigned int y;
    Point(unsigned int x, unsigned int y) : x(x), y(y) {}
  };

  struct Size {
    unsigned int width;
    unsigned int height;
    Size(unsigned int width, unsigned int height)
        : width(width), height(height) {}
  };

  Win32Window();
  virtual ~Win32Window();

  // Creates a win32 window with |title| that is positioned and sized using
  // |origin| and |size|. New windows are created on the default monitor. Window
  // sizes are specified to the OS in physical pixels, hence to ensure a
  // consistent size this function will scale the inputted width and height as
  // as appropriate for the default monitor. The window is invisible until
  // |Show| is called. Returns true if the window was created successfully.
  bool Create(const std::wstring& title, const Point& origin, const Size& size);

  // Show the current window. Returns true if the window was successfully shown.
  bool Show();

  // Release OS resources associated with window.
  void Destroy();

  // Inserts |content| into the window tree.
  void SetChildContent(HWND content);

  // Returns the backing Window handle to enable clients to set icon and other
  // window properties. Returns nullptr if the window has been destroyed.
  HWND GetHandle();

  // If true, closing this window will quit the application.
  void SetQuitOnClose(bool quit_on_close);

  // Return a RECT representing the bounds of the current client area.
  RECT GetClientArea();

 protected:
  // Processes and route salient window messages for mouse handling,
  // size change and DPI. Delegates handling of these to member overloads that
  // inheriting classes can handle.
  virtual LRESULT MessageHandler(HWND window,
                                 UINT const message,
                                 WPARAM const wparam,
                                 LPARAM const lparam) noexcept;

  // Called when CreateAndShow is called, allowing subclass window-related
  // setup. Subclasses should return false if setup fails.
  virtual bool OnCreate();

  // Called when Destroy is called.
  virtual void OnDestroy();

 private:
  friend class WindowClassRegistrar;

  // OS callback called by message pump. Handles the WM_NCCREATE message which
  // is passed when the non-client area is being created and enables automatic
  // non-client DPI scaling so that the non-client area automatically
  // responds to changes in DPI. All other messages are handled by
  // MessageHandler.
  static LRESULT CALLBACK WndProc(HWND const window,
                                  UINT const message,
                                  WPARAM const wparam,
                                  LPARAM const lparam) noexcept;

  // Retrieves a class instance pointer for |window|
  static Win32Window* GetThisFromHandle(HWND const window) noexcept;

  // Update the window frame's theme to match the system theme.
  static void UpdateTheme(HWND const window);

  bool quit_on_close_ = false;

  // window handle for top level window.
  HWND window_handle_ = nullptr;

  // window handle for hosted content.
  HWND child_content_ = nullptr;
};

#endif  // RUNNER_WIN32_WINDOW_H_

``````


### windows/runner/win32_window.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/win32_window.cpp
`relative_path`: windows/runner/win32_window.cpp
`format`: Arbitrary Binary Data
`size`: 8534   




### windows/runner/resources/app_icon.ico
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/resources/app_icon.ico
`relative_path`: windows/runner/resources/app_icon.ico
`format`: Windows Icon
`size`: 33772   




### windows/runner/resource.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/resource.h
`relative_path`: windows/runner/resource.h
`format`: Arbitrary Binary Data
`size`: 432   


``````
//{{NO_DEPENDENCIES}}
// Microsoft Visual C++ generated include file.
// Used by Runner.rc
//
#define IDI_APP_ICON                    101

// Next default values for new objects
//
#ifdef APSTUDIO_INVOKED
#ifndef APSTUDIO_READONLY_SYMBOLS
#define _APS_NEXT_RESOURCE_VALUE        102
#define _APS_NEXT_COMMAND_VALUE         40001
#define _APS_NEXT_CONTROL_VALUE         1001
#define _APS_NEXT_SYMED_VALUE           101
#endif
#endif

``````


### windows/runner/Runner.rc
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/Runner.rc
`relative_path`: windows/runner/Runner.rc
`format`: Arbitrary Binary Data
`size`: 3081   


``````
// Microsoft Visual C++ generated resource script.
//
#pragma code_page(65001)
#include "resource.h"

#define APSTUDIO_READONLY_SYMBOLS
/////////////////////////////////////////////////////////////////////////////
//
// Generated from the TEXTINCLUDE 2 resource.
//
#include "winres.h"

/////////////////////////////////////////////////////////////////////////////
#undef APSTUDIO_READONLY_SYMBOLS

/////////////////////////////////////////////////////////////////////////////
// English (United States) resources

#if !defined(AFX_RESOURCE_DLL) || defined(AFX_TARG_ENU)
LANGUAGE LANG_ENGLISH, SUBLANG_ENGLISH_US

#ifdef APSTUDIO_INVOKED
/////////////////////////////////////////////////////////////////////////////
//
// TEXTINCLUDE
//

1 TEXTINCLUDE
BEGIN
    "resource.h\0"
END

2 TEXTINCLUDE
BEGIN
    "#include ""winres.h""\r\n"
    "\0"
END

3 TEXTINCLUDE
BEGIN
    "\r\n"
    "\0"
END

#endif    // APSTUDIO_INVOKED


/////////////////////////////////////////////////////////////////////////////
//
// Icon
//

// Icon with lowest ID value placed first to ensure application icon
// remains consistent on all systems.
IDI_APP_ICON            ICON                    "resources\\app_icon.ico"


/////////////////////////////////////////////////////////////////////////////
//
// Version
//

#if defined(FLUTTER_VERSION_MAJOR) && defined(FLUTTER_VERSION_MINOR) && defined(FLUTTER_VERSION_PATCH) && defined(FLUTTER_VERSION_BUILD)
#define VERSION_AS_NUMBER FLUTTER_VERSION_MAJOR,FLUTTER_VERSION_MINOR,FLUTTER_VERSION_PATCH,FLUTTER_VERSION_BUILD
#else
#define VERSION_AS_NUMBER 1,0,0,0
#endif

#if defined(FLUTTER_VERSION)
#define VERSION_AS_STRING FLUTTER_VERSION
#else
#define VERSION_AS_STRING "1.0.0"
#endif

VS_VERSION_INFO VERSIONINFO
 FILEVERSION VERSION_AS_NUMBER
 PRODUCTVERSION VERSION_AS_NUMBER
 FILEFLAGSMASK VS_FFI_FILEFLAGSMASK
#ifdef _DEBUG
 FILEFLAGS VS_FF_DEBUG
#else
 FILEFLAGS 0x0L
#endif
 FILEOS VOS__WINDOWS32
 FILETYPE VFT_APP
 FILESUBTYPE 0x0L
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904e4"
        BEGIN
            VALUE "CompanyName", "com.example" "\0"
            VALUE "FileDescription", "vacuul_app_playground" "\0"
            VALUE "FileVersion", VERSION_AS_STRING "\0"
            VALUE "InternalName", "vacuul_app_playground" "\0"
            VALUE "LegalCopyright", "Copyright (C) 2024 com.example. All rights reserved." "\0"
            VALUE "OriginalFilename", "vacuul_app_playground.exe" "\0"
            VALUE "ProductName", "vacuul_app_playground" "\0"
            VALUE "ProductVersion", VERSION_AS_STRING "\0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1252
    END
END

#endif    // English (United States) resources
/////////////////////////////////////////////////////////////////////////////



#ifndef APSTUDIO_INVOKED
/////////////////////////////////////////////////////////////////////////////
//
// Generated from the TEXTINCLUDE 3 resource.
//


/////////////////////////////////////////////////////////////////////////////
#endif    // not APSTUDIO_INVOKED

``````


### windows/runner/main.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/main.cpp
`relative_path`: windows/runner/main.cpp
`format`: Arbitrary Binary Data
`size`: 1274   


``````
#include <flutter/dart_project.h>
#include <flutter/flutter_view_controller.h>
#include <windows.h>

#include "flutter_window.h"
#include "utils.h"

int APIENTRY wWinMain(_In_ HINSTANCE instance, _In_opt_ HINSTANCE prev,
                      _In_ wchar_t *command_line, _In_ int show_command) {
  // Attach to console when present (e.g., 'flutter run') or create a
  // new console when running with a debugger.
  if (!::AttachConsole(ATTACH_PARENT_PROCESS) && ::IsDebuggerPresent()) {
    CreateAndAttachConsole();
  }

  // Initialize COM, so that it is available for use in the library and/or
  // plugins.
  ::CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);

  flutter::DartProject project(L"data");

  std::vector<std::string> command_line_arguments =
      GetCommandLineArguments();

  project.set_dart_entrypoint_arguments(std::move(command_line_arguments));

  FlutterWindow window(project);
  Win32Window::Point origin(10, 10);
  Win32Window::Size size(1280, 720);
  if (!window.Create(L"vacuul_app_playground", origin, size)) {
    return EXIT_FAILURE;
  }
  window.SetQuitOnClose(true);

  ::MSG msg;
  while (::GetMessage(&msg, nullptr, 0, 0)) {
    ::TranslateMessage(&msg);
    ::DispatchMessage(&msg);
  }

  ::CoUninitialize();
  return EXIT_SUCCESS;
}

``````


### windows/runner/flutter_window.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/runner/flutter_window.h
`relative_path`: windows/runner/flutter_window.h
`format`: Arbitrary Binary Data
`size`: 928   


``````
#ifndef RUNNER_FLUTTER_WINDOW_H_
#define RUNNER_FLUTTER_WINDOW_H_

#include <flutter/dart_project.h>
#include <flutter/flutter_view_controller.h>

#include <memory>

#include "win32_window.h"

// A window that does nothing but host a Flutter view.
class FlutterWindow : public Win32Window {
 public:
  // Creates a new FlutterWindow hosting a Flutter view running |project|.
  explicit FlutterWindow(const flutter::DartProject& project);
  virtual ~FlutterWindow();

 protected:
  // Win32Window:
  bool OnCreate() override;
  void OnDestroy() override;
  LRESULT MessageHandler(HWND window, UINT const message, WPARAM const wparam,
                         LPARAM const lparam) noexcept override;

 private:
  // The project to run.
  flutter::DartProject project_;

  // The Flutter instance hosted by this window.
  std::unique_ptr<flutter::FlutterViewController> flutter_controller_;
};

#endif  // RUNNER_FLUTTER_WINDOW_H_

``````


### windows/flutter/generated_plugin_registrant.cc
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/flutter/generated_plugin_registrant.cc
`relative_path`: windows/flutter/generated_plugin_registrant.cc
`format`: Arbitrary Binary Data
`size`: 483   


``````
//
//  Generated file. Do not edit.
//

// clang-format off

#include "generated_plugin_registrant.h"

#include <url_launcher_windows/url_launcher_windows.h>
#include <window_to_front/window_to_front_plugin.h>

void RegisterPlugins(flutter::PluginRegistry* registry) {
  UrlLauncherWindowsRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("UrlLauncherWindows"));
  WindowToFrontPluginRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("WindowToFrontPlugin"));
}

``````


### windows/flutter/CMakeLists.txt
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/flutter/CMakeLists.txt
`relative_path`: windows/flutter/CMakeLists.txt
`format`: Arbitrary Binary Data
`size`: 3742   


``````
# This file controls Flutter-level build steps. It should not be edited.
cmake_minimum_required(VERSION 3.14)

set(EPHEMERAL_DIR "${CMAKE_CURRENT_SOURCE_DIR}/ephemeral")

# Configuration provided via flutter tool.
include(${EPHEMERAL_DIR}/generated_config.cmake)

# TODO: Move the rest of this into files in ephemeral. See
# https://github.com/flutter/flutter/issues/57146.
set(WRAPPER_ROOT "${EPHEMERAL_DIR}/cpp_client_wrapper")

# Set fallback configurations for older versions of the flutter tool.
if (NOT DEFINED FLUTTER_TARGET_PLATFORM)
  set(FLUTTER_TARGET_PLATFORM "windows-x64")
endif()

# === Flutter Library ===
set(FLUTTER_LIBRARY "${EPHEMERAL_DIR}/flutter_windows.dll")

# Published to parent scope for install step.
set(FLUTTER_LIBRARY ${FLUTTER_LIBRARY} PARENT_SCOPE)
set(FLUTTER_ICU_DATA_FILE "${EPHEMERAL_DIR}/icudtl.dat" PARENT_SCOPE)
set(PROJECT_BUILD_DIR "${PROJECT_DIR}/build/" PARENT_SCOPE)
set(AOT_LIBRARY "${PROJECT_DIR}/build/windows/app.so" PARENT_SCOPE)

list(APPEND FLUTTER_LIBRARY_HEADERS
  "flutter_export.h"
  "flutter_windows.h"
  "flutter_messenger.h"
  "flutter_plugin_registrar.h"
  "flutter_texture_registrar.h"
)
list(TRANSFORM FLUTTER_LIBRARY_HEADERS PREPEND "${EPHEMERAL_DIR}/")
add_library(flutter INTERFACE)
target_include_directories(flutter INTERFACE
  "${EPHEMERAL_DIR}"
)
target_link_libraries(flutter INTERFACE "${FLUTTER_LIBRARY}.lib")
add_dependencies(flutter flutter_assemble)

# === Wrapper ===
list(APPEND CPP_WRAPPER_SOURCES_CORE
  "core_implementations.cc"
  "standard_codec.cc"
)
list(TRANSFORM CPP_WRAPPER_SOURCES_CORE PREPEND "${WRAPPER_ROOT}/")
list(APPEND CPP_WRAPPER_SOURCES_PLUGIN
  "plugin_registrar.cc"
)
list(TRANSFORM CPP_WRAPPER_SOURCES_PLUGIN PREPEND "${WRAPPER_ROOT}/")
list(APPEND CPP_WRAPPER_SOURCES_APP
  "flutter_engine.cc"
  "flutter_view_controller.cc"
)
list(TRANSFORM CPP_WRAPPER_SOURCES_APP PREPEND "${WRAPPER_ROOT}/")

# Wrapper sources needed for a plugin.
add_library(flutter_wrapper_plugin STATIC
  ${CPP_WRAPPER_SOURCES_CORE}
  ${CPP_WRAPPER_SOURCES_PLUGIN}
)
apply_standard_settings(flutter_wrapper_plugin)
set_target_properties(flutter_wrapper_plugin PROPERTIES
  POSITION_INDEPENDENT_CODE ON)
set_target_properties(flutter_wrapper_plugin PROPERTIES
  CXX_VISIBILITY_PRESET hidden)
target_link_libraries(flutter_wrapper_plugin PUBLIC flutter)
target_include_directories(flutter_wrapper_plugin PUBLIC
  "${WRAPPER_ROOT}/include"
)
add_dependencies(flutter_wrapper_plugin flutter_assemble)

# Wrapper sources needed for the runner.
add_library(flutter_wrapper_app STATIC
  ${CPP_WRAPPER_SOURCES_CORE}
  ${CPP_WRAPPER_SOURCES_APP}
)
apply_standard_settings(flutter_wrapper_app)
target_link_libraries(flutter_wrapper_app PUBLIC flutter)
target_include_directories(flutter_wrapper_app PUBLIC
  "${WRAPPER_ROOT}/include"
)
add_dependencies(flutter_wrapper_app flutter_assemble)

# === Flutter tool backend ===
# _phony_ is a non-existent file to force this command to run every time,
# since currently there's no way to get a full input/output list from the
# flutter tool.
set(PHONY_OUTPUT "${CMAKE_CURRENT_BINARY_DIR}/_phony_")
set_source_files_properties("${PHONY_OUTPUT}" PROPERTIES SYMBOLIC TRUE)
add_custom_command(
  OUTPUT ${FLUTTER_LIBRARY} ${FLUTTER_LIBRARY_HEADERS}
    ${CPP_WRAPPER_SOURCES_CORE} ${CPP_WRAPPER_SOURCES_PLUGIN}
    ${CPP_WRAPPER_SOURCES_APP}
    ${PHONY_OUTPUT}
  COMMAND ${CMAKE_COMMAND} -E env
    ${FLUTTER_TOOL_ENVIRONMENT}
    "${FLUTTER_ROOT}/packages/flutter_tools/bin/tool_backend.bat"
      ${FLUTTER_TARGET_PLATFORM} $<CONFIG>
  VERBATIM
)
add_custom_target(flutter_assemble DEPENDS
  "${FLUTTER_LIBRARY}"
  ${FLUTTER_LIBRARY_HEADERS}
  ${CPP_WRAPPER_SOURCES_CORE}
  ${CPP_WRAPPER_SOURCES_PLUGIN}
  ${CPP_WRAPPER_SOURCES_APP}
)

``````


### windows/flutter/generated_plugins.cmake
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/flutter/generated_plugins.cmake
`relative_path`: windows/flutter/generated_plugins.cmake
`format`: Arbitrary Binary Data
`size`: 784   


``````
#
# Generated file, do not edit.
#

list(APPEND FLUTTER_PLUGIN_LIST
  url_launcher_windows
  window_to_front
)

list(APPEND FLUTTER_FFI_PLUGIN_LIST
)

set(PLUGIN_BUNDLED_LIBRARIES)

foreach(plugin ${FLUTTER_PLUGIN_LIST})
  add_subdirectory(flutter/ephemeral/.plugin_symlinks/${plugin}/windows plugins/${plugin})
  target_link_libraries(${BINARY_NAME} PRIVATE ${plugin}_plugin)
  list(APPEND PLUGIN_BUNDLED_LIBRARIES $<TARGET_FILE:${plugin}_plugin>)
  list(APPEND PLUGIN_BUNDLED_LIBRARIES ${${plugin}_bundled_libraries})
endforeach(plugin)

foreach(ffi_plugin ${FLUTTER_FFI_PLUGIN_LIST})
  add_subdirectory(flutter/ephemeral/.plugin_symlinks/${ffi_plugin}/windows plugins/${ffi_plugin})
  list(APPEND PLUGIN_BUNDLED_LIBRARIES ${${ffi_plugin}_bundled_libraries})
endforeach(ffi_plugin)

``````


### windows/flutter/generated_plugin_registrant.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul_app_playground/windows/flutter/generated_plugin_registrant.h
`relative_path`: windows/flutter/generated_plugin_registrant.h
`format`: Arbitrary Binary Data
`size`: 302   


``````
//
//  Generated file. Do not edit.
//

// clang-format off

#ifndef GENERATED_PLUGIN_REGISTRANT_
#define GENERATED_PLUGIN_REGISTRANT_

#include <flutter/plugin_registry.h>

// Registers Flutter plugins.
void RegisterPlugins(flutter::PluginRegistry* registry);

#endif  // GENERATED_PLUGIN_REGISTRANT_

``````



## vacuul-machine-firmware
`clone_url`: https://github.com/vacuul-dev/vacuul-machine-firmware.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 65   


``````
# vacuul-machine-firmware

Latest firmware received at 07/01/2025
``````


### src/middleware/curlThread.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/middleware/curlThread.cpp
`relative_path`: src/middleware/curlThread.cpp
`format`: Arbitrary Binary Data
`size`: 11699   




### src/middleware/thread.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/middleware/thread.h
`relative_path`: src/middleware/thread.h
`format`: Arbitrary Binary Data
`size`: 1013   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_THREAD_H
#define IG_MIDDLEWARE_THREAD_H

#include <cstddef>
#include <cstdint>
#include <mutex>


namespace thread {

class SharedData
{
public:
    using lock_guard = std::lock_guard<std::mutex>;

public:
    SharedData()
        : m_booted(false), m_terminate(false)
    {}

    virtual ~SharedData() {}

    // clang-format off
    void setBooted(bool state = true) { lock_guard lg(m_mtxThCtrl); m_booted = state; }
    void terminate(bool state = true) { lock_guard lg(m_mtxThCtrl); m_terminate = state; }
    bool isBooted() const { lock_guard lg(m_mtxThCtrl); return m_booted; }
    bool testTerminate() const { lock_guard lg(m_mtxThCtrl); return m_terminate; }
    // clang-format on

protected:
    mutable std::mutex m_mtx;
    bool m_booted;
    bool m_terminate;

private:
    mutable std::mutex m_mtxThCtrl;
};

} // namespace thread


#endif // IG_MIDDLEWARE_THREAD_H

``````


### src/middleware/log.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/middleware/log.h
`relative_path`: src/middleware/log.h
`format`: Arbitrary Binary Data
`size`: 3525   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_LOG_H
#define IG_MIDDLEWARE_LOG_H

#include <cstdio>



#define LOG_LEVEL_OFF (0)
#define LOG_LEVEL_ERR (1)
#define LOG_LEVEL_WRN (2)
#define LOG_LEVEL_INF (3)
#define LOG_LEVEL_DBG (4)

#ifndef CONFIG_LOG_LEVEL
#warning "CONFIG_LOG_LEVEL is not defined, defaulting to 2 (warning)"
#define CONFIG_LOG_LEVEL LOG_LEVEL_WRN
#endif

#ifndef LOG_MODULE_LEVEL
#error "define LOG_MODULE_LEVEL before including log.h"
#endif
#ifndef LOG_MODULE_NAME
#error "define LOG_MODULE_NAME before including log.h"
#endif



// SGR foreground colors
#define LOG_SGR_BLACK    "\033[30m"
#define LOG_SGR_RED      "\033[31m"
#define LOG_SGR_GREEN    "\033[32m"
#define LOG_SGR_YELLOW   "\033[33m"
#define LOG_SGR_BLUE     "\033[34m"
#define LOG_SGR_MAGENTA  "\033[35m"
#define LOG_SGR_CYAN     "\033[36m"
#define LOG_SGR_WHITE    "\033[37m"
#define LOG_SGR_DEFAULT  "\033[39m"
#define LOG_SGR_BBLACK   "\033[90m"
#define LOG_SGR_BRED     "\033[91m"
#define LOG_SGR_BGREEN   "\033[92m"
#define LOG_SGR_BYELLOW  "\033[93m"
#define LOG_SGR_BBLUE    "\033[94m"
#define LOG_SGR_BMAGENTA "\033[95m"
#define LOG_SGR_BCYAN    "\033[96m"
#define LOG_SGR_BWHITE   "\033[97m"



// optional args
#define ___LOG_OPT_VA_ARGS(...) , ##__VA_ARGS__

// stringify
#define ___LOG_STR_HELPER(x) #x
#define ___LOG_STR(x)        ___LOG_STR_HELPER(x)

#define ___LOG_CSI_EL "\033[2K" // ANSI ESC CSI erase line



// config can limit log level
#if (CONFIG_LOG_LEVEL < LOG_MODULE_LEVEL)
#undef LOG_MODULE_LEVEL
#define LOG_MODULE_LEVEL CONFIG_LOG_LEVEL
#endif



#include "middleware/util.h"

// clang-format off
#define LOG_ERR(msg, ...) printf((___LOG_CSI_EL "[" + util::t_to_iso8601_local(std::time(nullptr)) + "] " "\033[91m" ___LOG_STR(LOG_MODULE_NAME) " <ERR> " msg "\033[39m" "\n").c_str() ___LOG_OPT_VA_ARGS(__VA_ARGS__))
#define LOG_WRN(msg, ...) printf((___LOG_CSI_EL "[" + util::t_to_iso8601_local(std::time(nullptr)) + "] " "\033[93m" ___LOG_STR(LOG_MODULE_NAME) " <WRN> " msg "\033[39m" "\n").c_str() ___LOG_OPT_VA_ARGS(__VA_ARGS__))
#define LOG_INF(msg, ...) printf((___LOG_CSI_EL "[" + util::t_to_iso8601_local(std::time(nullptr)) + "] " "\033[39m" ___LOG_STR(LOG_MODULE_NAME) " <INF> " msg "\033[39m" "\n").c_str() ___LOG_OPT_VA_ARGS(__VA_ARGS__))
//#define LOG_DBG(msg, ...) printf((___LOG_CSI_EL "[" + util::t_to_iso8601_local(std::time(nullptr)) + "] " "\033[39m" ___LOG_STR(LOG_MODULE_NAME) " <DBG> " msg "\033[39m" "\n").c_str() ___LOG_OPT_VA_ARGS(__VA_ARGS__))
#define LOG_DBG(msg, ...) printf((___LOG_CSI_EL "[" + util::t_to_iso8601_local(std::time(nullptr)) + "] " "\033[39m" ___LOG_STR(LOG_MODULE_NAME) " <DBG> \033[90m" + std::string(__func__) +  "():" + std::to_string(__LINE__) + "\033[39m " msg "\033[39m" "\n").c_str() ___LOG_OPT_VA_ARGS(__VA_ARGS__))
// clang-format on



#if (LOG_MODULE_LEVEL < LOG_LEVEL_DBG)
#undef LOG_DBG
#define LOG_DBG(...) (void)0
#endif
#if (LOG_MODULE_LEVEL < LOG_LEVEL_INF)
#undef LOG_INF
#define LOG_INF(...) (void)0
#endif
#if (LOG_MODULE_LEVEL < LOG_LEVEL_WRN)
#undef LOG_WRN
#define LOG_WRN(...) (void)0
#endif
#if (LOG_MODULE_LEVEL < LOG_LEVEL_ERR)
#undef LOG_ERR
#define LOG_ERR(...) (void)0
#endif



#include <unistd.h>
#define LOG_invalidState(_state, _t_s)       \
    {                                        \
        LOG_ERR("invalid state %i", _state); \
        usleep(_t_s * 1000 * 1000);          \
    }
// end LOG_invalidState



#endif // IG_MIDDLEWARE_LOG_H

``````


### src/middleware/json.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/middleware/json.h
`relative_path`: src/middleware/json.h
`format`: Arbitrary Binary Data
`size`: 2669   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_JSON_H
#define IG_MIDDLEWARE_JSON_H



// #pragma GCC diagnostic push
// #pragma GCC diagnostic ignored "-Wpsabi" // does not help :(
//                                             needed to add it in CMake:
//                                             target_compile_options(${BINNAME} PRIVATE -Wno-psabi -W...)

// https://stackoverflow.com/a/48149400
#include <json/json.hpp>

// #pragma GCC diagnostic pop



using json = nlohmann::json;



class JsonTypeCheckItem
{
public:
    JsonTypeCheckItem() = delete;

    JsonTypeCheckItem(const std::string& key, const json::value_t& type)
        : m_key(key), m_nDiffTypes(1), m_type0(type), m_type1(type), m_type2(type)
    {}

    JsonTypeCheckItem(const std::string& key, const json::value_t& type0, const json::value_t& type1)
        : m_key(key), m_nDiffTypes(2), m_type0(type0), m_type1(type1), m_type2(type1)
    {}

    JsonTypeCheckItem(const std::string& key, const json::value_t& type0, const json::value_t& type1, const json::value_t& type2)
        : m_key(key), m_nDiffTypes(3), m_type0(type0), m_type1(type1), m_type2(type2)
    {}

    const std::string& key() const { return m_key; }
    size_t nDiffTypes() const { return m_nDiffTypes; }
    const json::value_t& type0() const { return m_type0; }
    const json::value_t& type1() const { return m_type1; }
    const json::value_t& type2() const { return m_type2; }

    bool checkType(const json::value_t& type) const { return ((type == m_type0) || (type == m_type1) || (type == m_type2)); }

private:
    std::string m_key;
    size_t m_nDiffTypes;
    json::value_t m_type0;
    json::value_t m_type1;
    json::value_t m_type2;
};



std::string jsonTypeCheck(const json& data, const JsonTypeCheckItem& check);

/**
 * @brief
 *
 * @param data The parent JSON object
 * @param key Key of the value to be checked
 * @param type Expected type
 * @return Exception message, empty if OK
 */
static inline std::string jsonTypeCheck(const json& data, const std::string& key, json::value_t type)
{
    return jsonTypeCheck(data, JsonTypeCheckItem(key, type));
}

static inline std::string jsonTypeCheck(const json& data, const std::string& key, json::value_t type0, json::value_t type1)
{
    return jsonTypeCheck(data, JsonTypeCheckItem(key, type0, type1));
}

static inline std::string jsonTypeCheck(const json& data, const std::string& key, json::value_t type0, json::value_t type1, json::value_t type2)
{
    return jsonTypeCheck(data, JsonTypeCheckItem(key, type0, type1, type2));
}


#endif // IG_MIDDLEWARE_JSON_H

``````


### src/middleware/json.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/middleware/json.cpp
`relative_path`: src/middleware/json.cpp
`format`: Arbitrary Binary Data
`size`: 1204   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <string>

#include "json.h"


namespace {}



std::string jsonTypeCheck(const json& data, const JsonTypeCheckItem& check)
{
    std::string exWhat;

    try
    {
        const auto value = data.at(check.key());

        if (false == check.checkType(value.type()))
        {
            const auto n = check.nDiffTypes();
            const auto j0 = json(check.type0());
            const auto j1 = json(check.type1());
            const auto j2 = json(check.type2());

            std::string expectedStr = "expected " + std::string(j0.type_name());
            if (n > 1) { expectedStr += " or " + std::string(j1.type_name()); }
            if (n > 2) { expectedStr += " or " + std::string(j2.type_name()); }

            throw std::runtime_error("\"" + check.key() + "\" is " + std::string(value.type_name()) + " (" + expectedStr + ")");
        }
    }
    catch (const std::exception& ex)
    {
        exWhat = ex.what();
    }
    catch (...)
    {
        exWhat = std::string(__func__) + " unknown exception";
    }

    return exWhat;
}

``````


### src/middleware/curlThread.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/middleware/curlThread.h
`relative_path`: src/middleware/curlThread.h
`format`: Arbitrary Binary Data
`size`: 6148   




### src/api/api.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/api/api.cpp
`relative_path`: src/api/api.cpp
`format`: Arbitrary Binary Data
`size`: 21264   




### src/api/data.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/api/data.h
`relative_path`: src/api/data.h
`format`: Arbitrary Binary Data
`size`: 4022   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_API_DATA_H
#define IG_API_DATA_H

#include <cstddef>
#include <cstdint>
#include <string>

#include "application/treatmentData.h"


namespace api {

class req_data_base
{
public:
    req_data_base() {}
    virtual ~req_data_base() {}

    // aka serialise
    virtual std::string httpBody() const = 0;
};

class res_data_base
{
public:
    res_data_base() {}
    virtual ~res_data_base() {}

    virtual void set(const std::string& httpBody) noexcept(false) = 0;
};



//======================================================================================================================
// requests

class CommissionReq : public req_data_base // TODO is a derived class really needed? comission id is already in appdata in the default req JSON
{
public:
    class ID
    {
    public:
        ID() = delete;

        explicit ID(const std::string& id)
            : m_id(id)
        {}

        const std::string& get() const { return m_id; }

    private:
        std::string m_id;
    };

public:
    CommissionReq()
        : req_data_base(), m_id()
    {}

    CommissionReq(const ID& id)
        : req_data_base(), m_id(id.get())
    {}

    virtual ~CommissionReq() {}

    virtual std::string httpBody() const;

private:
    std::string m_id;
};

class StartReq : public req_data_base
{
public:
    StartReq()
        : req_data_base()
    {}

    virtual ~StartReq() {}

    virtual std::string httpBody() const;
};

class SettingsReq : public req_data_base
{
public:
    SettingsReq()
        : req_data_base()
    {}

    virtual ~SettingsReq() {}

    virtual std::string httpBody() const;
};

class ProgressReq : public req_data_base
{
public:
    ProgressReq()
        : req_data_base()
    {}

    virtual ~ProgressReq() {}

    virtual std::string httpBody() const;
};

class ErrorReq : public req_data_base
{
public:
    ErrorReq()
        : req_data_base()
    {}

    virtual ~ErrorReq() {}

    virtual std::string httpBody() const;
};



//======================================================================================================================
// responses

class CommissionRes : public res_data_base
{
public:
    CommissionRes()
        : res_data_base(), m_dataReady(false), m_machineId(), m_timezone(), m_wifiCountry()
    {}

    virtual ~CommissionRes() {}

    bool dataReady() const { return m_dataReady; }

    const std::string& machineId() const { return m_machineId; }
    const std::string& timezone() const { return m_timezone; }
    const std::string& wifiCountry() const { return m_wifiCountry; }

    virtual void set(const std::string& httpBody) noexcept(false);

private:
    bool m_dataReady;
    std::string m_machineId;
    std::string m_timezone;
    std::string m_wifiCountry;
};

class StartRes : public res_data_base
{
public:
    StartRes()
        : res_data_base(), m_treatClearance(false)
    {}

    virtual ~StartRes() {}

    bool treatClearance() const { return m_treatClearance; }

    virtual void set(const std::string& httpBody) noexcept(false);

private:
    bool m_treatClearance;
};

class SettingsRes : public res_data_base
{
public:
    SettingsRes()
        : res_data_base(), m_user("<nickname>"), m_treatConfig(app::treat::Config::blocks_type())
    {}

    virtual ~SettingsRes() {}

    const app::treat::User& user() const { return m_user; }
    const app::treat::Config& treatConfig() const { return m_treatConfig; }

    virtual void set(const std::string& httpBody) noexcept(false);

private:
    app::treat::User m_user;
    app::treat::Config m_treatConfig;
};

class ProgressRes : public res_data_base
{
public:
    ProgressRes()
        : res_data_base(), m_abort(false)
    {}

    virtual ~ProgressRes() {}

    bool abortTreatment() const { return m_abort; }

    virtual void set(const std::string& httpBody) noexcept(false);

private:
    bool m_abort;
};

} // namespace api


#endif // IG_API_DATA_H

``````


### src/api/data.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/api/data.cpp
`relative_path`: src/api/data.cpp
`format`: Arbitrary Binary Data
`size`: 8719   




### src/api/api.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/api/api.h
`relative_path`: src/api/api.h
`format`: Arbitrary Binary Data
`size`: 4573   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

/*

data:
 - set req data (one per endpoint)
 - get res data (one per endpoint)
 - status
    - state (idle, req, res, error)
    - current endpoint (don't care in idle state)
    - timestamps (req, res)
    - duration

Serialising of the req data and parsing of the res data is explicitly done in the API thread.

controlling / thread sync:
 - set req data triggers the request, returns with no effect if state != idle
 - get res data sets state to idle
 - flush method to set state to idle (in case of error), returns with no effect if state is neither error nor res

*/

#ifndef IG_API_API_H
#define IG_API_API_H

#include <cstddef>
#include <cstdint>
#include <ctime>
#include <string>

#include "api/data.h"
#include "middleware/thread.h"
#include "middleware/util.h"


namespace api::thread {

typedef enum
{
    ep_commission,
    ep_start,    // start treatment clearance
    ep_settings, // treatment/user config
    ep_progress, // report treatment progress/data
    ep_error,

    ep__end_
} endpoint_t;

typedef enum STATE
{
    state_idle,  // ready to make a request
    state_req,   // request is ongoing
    state_res,   // response is ready
    state_error, // an error occured

    state__end_
} state_t;

class Status
{
public:
    Status()
        : m_state(state_idle), m_ep(), m_tReq(), m_tRes(), m_dur(0)
    {}

    virtual ~Status() {}

    int reqEndpoint(endpoint_t ep);
    void resGotten(endpoint_t ep);
    void setState(state_t state) { m_state = state; }
    void setTReq(time_t t) { m_tReq = t; }
    void setTRes(time_t t) { m_tRes = t; }
    void setDuration(omw_::clock::timepoint_t dur_us) { m_dur = dur_us; }

    state_t state() const { return m_state; }
    endpoint_t endpoint() const { return m_ep; }
    time_t tReq() const { return m_tReq; }
    time_t tRes() const { return m_tRes; }
    omw_::clock::timepoint_t duration() const { return m_dur; }

private:
    state_t m_state;
    endpoint_t m_ep;
    time_t m_tReq;
    time_t m_tRes;
    omw_::clock::timepoint_t m_dur;
};



std::string toString(endpoint_t ep);



class ThreadSharedData : public ::thread::SharedData
{
public:
    ThreadSharedData()
        : m_status()
    {}

    virtual ~ThreadSharedData() {}



    // extern
public:
    int reqCommission(const CommissionReq& data);
    int reqStart(const StartReq& data);
    int reqSettings(const SettingsReq& data);
    int reqProgress(const ProgressReq& data);
    int reqError(const ErrorReq& data);

    CommissionRes getCommissionRes() const;
    StartRes getStartRes() const;
    SettingsRes getSettingsRes() const;
    ProgressRes getProgressRes() const;

    void flush() const;

    // clang-format off
    Status status() const { lock_guard lg(m_mtx); return m_status; }
    // clang-format on



    // intern
public:
    // clang-format off
    void setState(state_t state)    { lock_guard lg(m_mtx); m_status.setState(state); }
    void setTReq(time_t t)          { lock_guard lg(m_mtx); m_status.setTReq(t); }
    void setTRes(time_t t)          { lock_guard lg(m_mtx); m_status.setTRes(t); }
    void setDur(omw_::clock::timepoint_t dur_us) { lock_guard lg(m_mtx); m_status.setDuration(dur_us); }

    void setCommissionRes(const CommissionRes& data) { lock_guard lg(m_mtx); m_commissionRes = data; }
    void setStartRes(const StartRes& data) { lock_guard lg(m_mtx); m_startRes = data; }
    void setSettingsRes(const SettingsRes& data) { lock_guard lg(m_mtx); m_settingsRes = data; }
    void setProgressRes(const ProgressRes& data) { lock_guard lg(m_mtx); m_progressRes = data; }

    CommissionReq getCommissionReq() const { lock_guard lg(m_mtx); return m_commissionReq; }
    StartReq getStartReq() const { lock_guard lg(m_mtx); return m_startReq; }
    SettingsReq getSettingsReq() const { lock_guard lg(m_mtx); return m_settingsReq; }
    ProgressReq getProgressReq() const { lock_guard lg(m_mtx); return m_progressReq; }
    ErrorReq getErrorReq() const { lock_guard lg(m_mtx); return m_errorReq; }
    // clang-format on



private:
    mutable Status m_status;
    CommissionReq m_commissionReq;
    CommissionRes m_commissionRes;
    StartReq m_startReq;
    StartRes m_startRes;
    SettingsReq m_settingsReq;
    SettingsRes m_settingsRes;
    ProgressReq m_progressReq;
    ProgressRes m_progressRes;
    ErrorReq m_errorReq;
};

extern ThreadSharedData sd;

void thread();

time_t getReqInterval(time_t min, time_t max);

} // namespace api::thread


#endif // IG_API_API_H

``````


### src/project.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/project.h
`relative_path`: src/project.h
`format`: Arbitrary Binary Data
`size`: 1325   


``````
/*
author          Oliver Blaser
date            22.05.2024
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_PROJECT_H
#define IG_PROJECT_H

#include <omw/defs.h>
#include <omw/version.h>


#define PRJ_ENTER_LOCALSETUP_FROM_IDLE (1)
#define PRJ_EN_PREPARE_STATE           (0)
#define PRJ_USE_HTML_IO_EMU            (0)
#define PRJ_START_AT_HANDS_ON          (1)

#define PRJ_PKG_DEV_DISPDMY (0)


#if PRJ_PKG_DEV_DISPDMY
#define PRJ_VERSION_BUILD ("DEV-DISPDMY")
#elif defined(_DEBUG)
#define PRJ_VERSION_BUILD ("DEBUG")
#endif // PRJ_PKG_DEV_DISPDMY

namespace prj {

const char* const appDirName = "vacuul"; // eq to package name

const char* const appName = "Vacuul";
const char* const binName = "vacuul"; // eq to the linker setting


#ifndef PRJ_VERSION_BUILD
#define PRJ_VERSION_BUILD ("")
#endif
const omw::Version version(0, 2, 10, "alpha", PRJ_VERSION_BUILD);

// const char* const website = "https://silab.ch/";

} // namespace prj


#ifdef OMW_DEBUG
#define PRJ_DEBUG (1)
#else
#undef PRJ_DEBUG
#endif



#ifndef RPIHAL_EMU

#ifndef __linux__
#error "not a Linux platform"
#endif

#if (!defined(__arm__) && !defined(__aarch64__))
#error "not an ARM platform" // nothing really ARM speciffic is used, just to detect RasPi
#endif

#endif // RPIHAL_EMU


#endif // IG_PROJECT_H

``````


### src/application/treatment.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/treatment.cpp
`relative_path`: src/application/treatment.cpp
`format`: Arbitrary Binary Data
`size`: 15393   




### src/application/commissioning.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/commissioning.cpp
`relative_path`: src/application/commissioning.cpp
`format`: Arbitrary Binary Data
`size`: 4394   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ctime>

#include "api/api.h"
#include "application/config.h"
#include "application/status.h"
#include "commissioning.h"

#include <omw/string.h>
#include <unistd.h>


#define LOG_MODULE_LEVEL LOG_LEVEL_DBG
#define LOG_MODULE_NAME  COMIS
#include "middleware/log.h"


namespace {

enum
{
    S_init = 0,
    S_idle,
    S_req,
    S_awaitRes,
    S_process,
    S_retry,
    S_done,
};

std::string generateCommissionId()
{
    std::string str;

    // const char* const vinDigits = "1234567890ABCDEFGHJKLMNPRSTUVWXYZ"; // VIN - vehicle identification number
    static const char digits[] = "1234567890ABCDEFGHJKLMNPRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    constexpr size_t digitsSize = SIZEOF_ARRAY(digits) - 1; // -1 because \0

    constexpr size_t digitsPerGrp = 4;
    constexpr size_t nGrps = 4;
    constexpr size_t len = (digitsPerGrp * nGrps);

    for (size_t i = 0; i < len; ++i)
    {
        if (((i % digitsPerGrp) == 0) && (i > 0)) { str += '-'; }

        str += *(digits + (rand() % digitsSize));
    }

    return str;
}

} // namespace



int app::commissioning::task()
{
    int r = commissioning::running;

    static int state = S_init;
    static std::string commissionId = "";
    static api::CommissionRes resData;
    static time_t tAttempt = 0;
    static size_t attemptCnt = 0;
    static time_t retryDelay = 0;

    const time_t tNow = time(nullptr);

    static int oldState = -1;
    if (oldState != state)
    {
        LOG_DBG("%s state %i -> %i", ___LOG_STR(LOG_MODULE_NAME), oldState, state);

        oldState = state;
    }

    switch (state)
    {
    case S_init:
        srand(tNow);
        commissionId = generateCommissionId();
        tAttempt = 0;
        retryDelay = 0;
        state = S_idle;
        break;

    case S_idle:
        if ((tNow - tAttempt) >= retryDelay)
        {
            tAttempt = tNow;
            state = S_req;
        }
        break;

    case S_req:
        if (api::thread::sd.reqCommission(api::CommissionReq::ID(commissionId) // TODO use real comis data class
                                          ) == 0)
        {
            LOG_DBG("req with commission ID: %s", commissionId.c_str());

            ++attemptCnt;
            app::appData.set(app::CommissionData(commissionId, attemptCnt));

            state = S_awaitRes;
        }
        break;

    case S_awaitRes:
    {
        const auto status = api::thread::sd.status();

        if (status.state() == api::thread::state_res)
        {
            resData = api::thread::sd.getCommissionRes();
            state = S_process;
        }
        else if (status.state() == api::thread::state_error)
        {
            api::thread::sd.flush();
            state = S_retry;
        }
    }
    break;

    case S_process:
        if (resData.dataReady())
        {
            const std::string machineId = resData.machineId();
            const std::string timezone = resData.timezone();
            const std::string wifiCountry = resData.wifiCountry();

            app::status.setMachineId(machineId);

            app::config.sys_wifiCountry = wifiCountry; // no more to do with the WiFi country (will be preset in local setup screen)
            app::config.sys_machineId = machineId;
            const int err = app::config.save();
            if (err)
            {
                LOG_ERR("failed to save config file (%i)", err);
                status.addDbgMsg("failed to save machine ID to config file (" + std::to_string(err) + ")");

                return commissioning::error;
            }

            // TODO set timezone in system

            LOG_INF("commissioning OK - machine ID: %s, timezone: %s", machineId.c_str(), timezone.c_str());

            state = S_done;
        }
        else
        {
            LOG_DBG("commission data not ready");
            state = S_retry;
        }
        break;

    case S_retry:
        retryDelay = api::thread::getReqInterval(3, 8);
        LOG_DBG("retryDelay: %is", (int)retryDelay);
        state = S_idle;
        break;

    case S_done:
        state = S_init;
        r = commissioning::done;
        break;

    default:
        LOG_invalidState(state, 5);
        break;
    }

    return r;
}

``````


### src/application/status.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/status.h
`relative_path`: src/application/status.h
`format`: Arbitrary Binary Data
`size`: 5508   




### src/application/config.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/config.h
`relative_path`: src/application/config.h
`format`: Arbitrary Binary Data
`size`: 1660   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_CONFIG_H
#define IG_APP_CONFIG_H

#include <cstddef>
#include <cstdint>
#include <ctime>
#include <string>
#include <string_view>
#include <vector>

#include "middleware/iniFile.h"
#include "project.h"

#include <omw/string.h>
#include <omw/version.h>


namespace app {

class Config
{
public:
    static constexpr std::string_view default_cfg_binVer = "0.0.0";
    static constexpr bool default_cfg_writeCfgFile = true;

    static constexpr std::string_view default_sys_machineId = "0";
    static constexpr int default_sys_displayBrightness = 100;
    static constexpr std::string_view default_sys_wifiCountry = ""; // empty by design

    // static constexpr bool default_ls_done = false;
    // static constexpr std::string_view default_ls_state = "-";

    static constexpr std::string_view default_api_baseUrl = "https://3.70.228.144:3000/";

public:
    Config();

    omw::Version cfg_binVer;
    bool cfg_writeCfgFile;

    std::string sys_machineId;
    int sys_displayBrightness;
    std::string sys_wifiCountry;

    // bool ls_done;
    // std::string ls_state;

    std::string api_baseUrl;

    void setFileName(const std::string& name) { iniFile.setFileName(name); }
    const std::string& getFileName() const { return iniFile.getFileName(); }

    int getUpdateResult() const;

    int save();
    int read();

    std::string dump() const { return iniFile.dump(); }

private:
    mw::IniFile iniFile;
    int updateResult;

    int update();
};

extern Config config;

} // namespace app


#endif // IG_APP_CONFIG_H

``````


### src/application/treatmentData.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/treatmentData.h
`relative_path`: src/application/treatmentData.h
`format`: Arbitrary Binary Data
`size`: 1862   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_TREATMENTDATA_H
#define IG_APP_TREATMENTDATA_H

#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>



namespace app::treat {

constexpr float defaultTreatmentStartTemp = 17.0f; // this temperature is held in idle

class Block
{
public:
    enum class Type
    {
        treat = 0,
        pause,
        end,
    };

public:
    Block() = delete;

    // Block(Block::Type type, uint32_t dur_s)
    //     : m_type(type), m_temp(defaultTreatmentStartTemp), m_dur(dur_s)
    //{}

    Block(Block::Type type, uint32_t dur_s, float temp)
        : m_type(type), m_temp(temp), m_dur(dur_s)
    {}

    virtual ~Block() {}

    Block::Type type() const { return m_type; }

    float temp() const { return m_temp; }

    // block duration [s]
    uint32_t duration() const { return m_dur; }

private:
    Block::Type m_type;
    float m_temp;   // peltier temperature [degC]
    uint32_t m_dur; // duration [s]
};

extern const Block endBlock;

class Config
{
public:
    using blocks_type = std::vector<app::treat::Block>;

public:
    Config() = delete; // explicitly init empty config

    Config(const blocks_type& blocks)
        : m_blocks(blocks)
    {}

    virtual ~Config() {}

    const Block& getBlock(blocks_type::size_type idx) const;

    // treatment duration [s]
    uint32_t duration() const;

private:
    blocks_type m_blocks;
};

class User
{
public:
    User(const std::string& nickname)
        : m_nickname(nickname)
    {}

    virtual ~User() {}

    const std::string& nickname() const { return m_nickname; }

private:
    std::string m_nickname;
    // user id (needed if offline logging (to RAM!) would be a thing in future)
};

} // namespace app::treat


#endif // IG_APP_TREATMENTDATA_H

``````


### src/application/status.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/status.cpp
`relative_path`: src/application/status.cpp
`format`: Arbitrary Binary Data
`size`: 3951   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>

#include "middleware/i2cThread.h"
#include "middleware/json.h"
#include "project.h"
#include "status.h"


#define LOG_MODULE_LEVEL LOG_LEVEL_INF
#define LOG_MODULE_NAME  STATUS
#include "middleware/log.h"


namespace {}


namespace app {

void AppData::set(const CommissionData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_commissionData = data;
    m_procData = &m_commissionData;
}

void AppData::set(const LocalSetupData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_localSetupData = data;
    m_procData = &m_localSetupData;
}

#ifdef ___CLASS_PrepareData_DECLARED
void AppData::set(const PrepareData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_prepareData = data;
    m_procData = &m_prepareData;
}
#endif // ___CLASS_PrepareData_DECLARED

void AppData::set(const IdleData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_idleData = data;
    m_procData = &m_idleData;
}

void AppData::set(const TreatmentData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_treatmentData = data;
    m_procData = &m_treatmentData;
}

void AppData::set(const ErrorData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_errorData = data;
    m_procData = &m_errorData;
}

bool AppData::handsOn() const
{
    bool r = false;

    if (&m_treatmentData == m_procData) { r = (static_cast<const TreatmentData*>(m_procData))->handsOn(); }

    return r;
}

std::string AppData::jsonDump()
{
    lock_guard lg(m_mtx);

    if (m_updateIdxJsonGen != m_updateIdx)
    {
        m_updateIdxJsonGen = m_updateIdx;

        if (m_procData) { m_procData->updateJsonDump(); }
    }

    json j;

    if (m_procData)
    {
        j = json::parse(m_procData->jsonDump());
        j["appState"] = m_procData->appStateStr();
    }
    else
    {
        j = json(json::value_t::object);
        j["appState"] = appStateStr::boot;
    }

    return j.dump();
}



std::string Status::jsonDump() const
{
    lock_guard lg(m_mtx);



    json devInfo(json::value_t::object);
    devInfo["swVersion"] = prj::version.toString();
    // devInfo["hwVersion"] = prj::hwv.toString();
    // devInfo["hwVersionName"] = prj::hwvDsipStr;

    json conn(json::value_t::object);
    conn["backend"] = m_backendConn;
    conn["internet"] = m_inetConn;

    json dbgInfoMax32664Info(json::value_t::object);
    dbgInfoMax32664Info["chipVersion"] = m_max32664Info.chipVersionStr();
    dbgInfoMax32664Info["version"] = m_max32664Info.version().toString();
    dbgInfoMax32664Info["afeType"] = m_max32664Info.afeTypeStr();
    dbgInfoMax32664Info["afeVersion"] = m_max32664Info.afeVersionStr();

    json dbgInfoMax32664(json::value_t::object);
    dbgInfoMax32664["info"] = dbgInfoMax32664Info;

    json dbgInfo(json::value_t::object);
    dbgInfo["threadBootFlags"] = m_thBootFlags;
    dbgInfo["peripheralErrorFlags"] = m_periphErrFlag;
    dbgInfo["max32664"] = dbgInfoMax32664;
    dbgInfo["messages"] = json(json::value_t::array);
    for (size_t i = 0; i < m_dbgmsg.size(); ++i)
    {
        json tmp(json::value_t::object);
        tmp["t"] = m_dbgmsg[i].time();
        tmp["msg"] = m_dbgmsg[i].msg();
        dbgInfo["messages"].push_back(tmp);
    }



    json j(json::value_t::object);
    j["deviceInfo"] = devInfo;
    j["machine_id"] = m_machineId;
    j["debugInfo"] = dbgInfo;
    j["airTemp"] = i2cThread::sd.getTempAir();
    j["connection"] = conn;



    return j.dump();
}

void Status::addDbgMsg(const std::string& msg)
{
    lock_guard lg(m_mtx);

    m_dbgmsg.push_back(DebugMessage(time(nullptr), msg));

    constexpr size_t maxNumOfMsg = 256;

    while (m_dbgmsg.size() > maxNumOfMsg) { m_dbgmsg.erase(m_dbgmsg.begin() + 0); }
}



app::AppData appData = app::AppData();

app::Status status = app::Status();

} // namespace app

``````


### src/application/idle.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/idle.h
`relative_path`: src/application/idle.h
`format`: Arbitrary Binary Data
`size`: 340   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_IDLE_H
#define IG_APP_IDLE_H

#include <cstddef>
#include <cstdint>


namespace app::idle {

enum
{
    running = 0,
    localSetup,
    startTreatment,

    ___ret_end_
};

int task();

}


#endif // IG_APP_IDLE_H

``````


### src/application/appMain.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/appMain.cpp
`relative_path`: src/application/appMain.cpp
`format`: Arbitrary Binary Data
`size`: 23903   




### src/application/dfu.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/dfu.cpp
`relative_path`: src/application/dfu.cpp
`format`: Arbitrary Binary Data
`size`: 15200   




### src/application/localSetup.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/localSetup.cpp
`relative_path`: src/application/localSetup.cpp
`format`: Arbitrary Binary Data
`size`: 25235   




### src/application/treatmentData.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/treatmentData.cpp
`relative_path`: src/application/treatmentData.cpp
`format`: Arbitrary Binary Data
`size`: 763   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>

#include "treatmentData.h"


#define LOG_MODULE_LEVEL LOG_LEVEL_INF
#define LOG_MODULE_NAME  TRETDATA
#include "middleware/log.h"


namespace {}



const app::treat::Block app::treat::endBlock(Block::Type::end, 0, app::treat::defaultTreatmentStartTemp);



namespace app::treat {

const Block& Config::getBlock(blocks_type::size_type idx) const
{
    if (idx < m_blocks.size()) { return m_blocks[idx]; }
    else { return app::treat::endBlock; }
}

uint32_t Config::duration() const
{
    uint32_t dur = 0;

    for (const auto& b : m_blocks) { dur += b.duration(); }

    return dur;
}

} // namespace app::treat

``````


### src/application/commissioning.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/commissioning.h
`relative_path`: src/application/commissioning.h
`format`: Arbitrary Binary Data
`size`: 361   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_COMMISSIONING_H
#define IG_APP_COMMISSIONING_H

#include <cstddef>
#include <cstdint>


namespace app::commissioning {

enum
{
    running = 0,
    done,
    error,

    ___ret_end_
};

int task();

}


#endif // IG_APP_COMMISSIONING_H

``````


### src/application/statusData.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/statusData.cpp
`relative_path`: src/application/statusData.cpp
`format`: Arbitrary Binary Data
`size`: 3604   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>

#include "middleware/json.h"
#include "project.h"
#include "statusData.h"


#define LOG_MODULE_LEVEL LOG_LEVEL_INF
#define LOG_MODULE_NAME  STATUSDATA
#include "middleware/log.h"


namespace {

json toJson(const app::PeltierStatusContainer& pelt)
{
    auto pstoj = [](const app::PeltierStatus& ps) {
        json j(json::value_t::object);
        j["temperature"] = ps.temp();
        j["setPoint"] = ps.setPoint();
        j["tolerance"] = ps.tolerance();
        return j;
    };

    json j(json::value_t::object);

    j["left"] = pstoj(pelt.left);
    j["right"] = pstoj(pelt.right);

    return j;
}

} // namespace



namespace app::appStateStr {

const char* const boot = "booting";
const char* const commission = "commissioning";
const char* const localSetup = "localSetup";
// const char* const prepare = "prepare";
const char* const idle = "idle";
const char* const treatment = "treatment";
const char* const error = "error";

}

namespace app {

ProcessData::ProcessData(const std::string& appStateStr)
    : m_jsonDump(), m_appStateStr(appStateStr)
{
    json j(json::value_t::object);
    m_jsonDump = j.dump();
}

void CommissionData::updateJsonDump()
{
    json j(json::value_t::object);

    j["commissionId"] = m_commissionId;
    j["attempt"] = m_attempt;

    m_jsonDump = j.dump();
}

void LocalSetupData::updateJsonDump()
{
    json system(json::value_t::object);
    system["displayBrightness"] = m_displayBrightness;

    json wifi(json::value_t::object);
    wifi["country"] = m_wifi_country;
    wifi["ssid"] = m_wifi_ssid;

    json j(json::value_t::object);
    j["system"] = system;
    j["wifi"] = wifi;

    m_jsonDump = j.dump();
}

#ifdef ___CLASS_PrepareData_DECLARED
void PrepareData::updateJsonDump()
{
    json j(json::value_t::object);

    j["peltier"] = toJson(m_pelt);

    m_jsonDump = j.dump();
}
#endif // ___CLASS_PrepareData_DECLARED

void IdleData::updateJsonDump()
{
    json j(json::value_t::object);

    m_jsonDump = j.dump();
}

void TreatmentData::set(bool handsOn, int noHandsTmr, uint32_t treatTime, uint32_t elapsedTime, const PeltierStatusContainer& pelt, const biometric::Data& biom)
{
    m_handsOn = handsOn;
    m_noHandsTmr = noHandsTmr;

    m_treatmentTime = treatTime;
    m_elapsedTime = elapsedTime;

    m_pelt = pelt;
    m_biom = biom;
}

void TreatmentData::updateJsonDump()
{
    json j(json::value_t::object);

    j["userName"] = m_userName;
    j["abortReason"] = m_abortReason;
    j["handsOn"] = m_handsOn;
    j["noHandsTimeout"] = m_noHandsTmr;
    j["treatmentDuration"] = m_treatmentDuration;
    j["treatmentTime"] = m_treatmentTime;
    j["elapsedTime"] = m_elapsedTime;

    switch (m_blockType)
    {
    case BlockType::init:
        j["blockType"] = "init";
        break;

    case BlockType::treat:
        j["blockType"] = "treat";
        break;

    case BlockType::pause:
        j["blockType"] = "pause";
        break;

    case BlockType::done:
        j["blockType"] = "done";
        break;
    }

    j["peltier"] = toJson(m_pelt);

    json biom(json::value_t::object);
    biom["heartRate"] = m_biom.heartRate();
    biom["oxygenSaturation"] = m_biom.oxygenSat();
    biom["algorithmState"] = m_biom.algorithmState();
    biom["algorithmStatus"] = m_biom.algorithmStatus();
    j["biometrics"] = biom;

    m_jsonDump = j.dump();
}

void ErrorData::updateJsonDump()
{
    json j(json::value_t::object);

    m_jsonDump = j.dump();
}

} // namespace app

``````


### src/application/idle.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/idle.cpp
`relative_path`: src/application/idle.cpp
`format`: Arbitrary Binary Data
`size`: 3564   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <ctime>

#include "api/api.h"
#include "application/appShared.h"
#include "application/status.h"
#include "application/treatment.h"
#include "idle.h"
#include "middleware/curlThread.h"
#include "middleware/i2cThread.h"
#include "middleware/util.h"
#include "project.h"


#define LOG_MODULE_LEVEL LOG_LEVEL_INF
#define LOG_MODULE_NAME  IDLE
#include "middleware/log.h"


namespace {

enum
{
    S_init = 0,
    S_enter,
    S_exit,
    S_idle,

    S_apiReqStart,
    S_apiAwaitResStart,

    S_apiReqSettings,
    S_apiAwaitResSettings,
};

}



int app::idle::task()
{
    int r = idle::running;

    static int state = S_init;
    static int returnCommand = idle::running;
    static time_t tAttempt = 0;
    static time_t retryDelay = 1;

    const time_t tNow = time(nullptr);

    static int oldState = -1;
    if (oldState != state)
    {
        LOG_DBG("%s state %i -> %i", ___LOG_STR(LOG_MODULE_NAME), oldState, state);

        oldState = state;
    }

    switch (state)
    {
    case S_init:
        state = S_enter;
        break;

    case S_enter:
        app::appData.set(app::IdleData());
        tAttempt = 0;

        state = S_idle;
        break;

    case S_exit:
        LOG_DBG("exit idle task: %i", returnCommand);
        state = S_enter;
        r = returnCommand;
        break;

    case S_idle:
        if ((tNow - tAttempt) >= retryDelay)
        {
#if PRJ_ENTER_LOCALSETUP_FROM_IDLE
            if (app::mouseConnected)
            {
                returnCommand = idle::localSetup;
                state = S_exit;
            }
            else
#endif
            {
                app::appData.set(app::IdleData());

                state = S_apiReqStart;
            }
        }
        break;

    case S_apiReqStart:
        if (api::thread::sd.reqStart(api::StartReq()) == 0) { state = S_apiAwaitResStart; }
        break;

    case S_apiAwaitResStart:
    {
        const auto status = api::thread::sd.status();

        if (status.state() == api::thread::state_res)
        {
            const api::StartRes resData = api::thread::sd.getStartRes();

            if (resData.treatClearance()) { state = S_apiReqSettings; }
            else { state = S_idle; }
        }
        else if (status.state() == api::thread::state_error)
        {
            api::thread::sd.flush();
            state = S_idle;
        }

#if defined(PRJ_DEBUG) && 1
        retryDelay = 1;
#else // PRJ_DEBUG

#if PRJ_START_AT_HANDS_ON
        retryDelay = 0;
#else
        retryDelay = api::thread::getReqInterval(2, 7);
#endif

#endif // PRJ_DEBUG
        tAttempt = tNow;
    }
    break;

    case S_apiReqSettings:
        if (api::thread::sd.reqSettings(api::SettingsReq()) == 0) { state = S_apiAwaitResSettings; }
        break;

    case S_apiAwaitResSettings:
    {
        const auto status = api::thread::sd.status();

        if (status.state() == api::thread::state_res)
        {
            const api::SettingsRes resData = api::thread::sd.getSettingsRes();

            app::treat::setConfig(resData.user(), resData.treatConfig());
            returnCommand = idle::startTreatment;
            state = S_exit;
        }
        else if (status.state() == api::thread::state_error)
        {
            api::thread::sd.flush();
            state = S_idle;
        }
    }
    break;

    default:
        LOG_invalidState(state, 5);
        break;
    }

    return r;
}

``````


### src/application/config.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/config.cpp
`relative_path`: src/application/config.cpp
`format`: Arbitrary Binary Data
`size`: 4598   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

#include "config.h"

#include <omw/string.h>


#define LOG_MODULE_LEVEL LOG_LEVEL_DBG
#define LOG_MODULE_NAME  CONFIG
#include "middleware/log.h"


namespace {

const char* const section_cfg = "Config";
const char* const key_cfg_binVer = "BinVer";
const char* const key_cfg_writeCfgFile = "WriteThisFileToDiskNextTime";

const char* const section_sys = "System";
const char* const key_sys_machineId = "MachineID";
const char* const key_sys_displayBrightness = "DisplayBrightness";
const char* const key_sys_wifiCountry = "WiFiCountry";

// const char* const section_ls = "LocalSetup";
// const char* const key_ls_done = "Done";
// const char* const key_ls_state = "State";

const char* const section_api = "API";
const char* const key_api_baseUrl = "BaseUrl";

} // namespace



app::Config::Config()
    : iniFile(), updateResult(0)
{
    iniFile.setLineEnding("\n");
    iniFile.setWriteBom(false);
}

int app::Config::getUpdateResult() const { return updateResult; }

int app::Config::save()
{
    iniFile.setValue(section_cfg, key_cfg_binVer, cfg_binVer.toString());
    iniFile.setValue(section_cfg, key_cfg_writeCfgFile, omw::to_string(cfg_writeCfgFile));

    iniFile.setValue(section_sys, key_sys_machineId, sys_machineId);
    iniFile.setValue(section_sys, key_sys_displayBrightness, omw::to_string(sys_displayBrightness));
    iniFile.setValue(section_sys, key_sys_wifiCountry, sys_wifiCountry);

    // iniFile.setValue(section_ls, key_ls_done, omw::to_string(ls_done));
    // iniFile.setValue(section_ls, key_ls_state, ls_state);

    iniFile.setValue(section_api, key_api_baseUrl, api_baseUrl);

    return iniFile.writeFile();
}

int app::Config::read()
{
    int r = iniFile.readFile();

    if (r == 0)
    {
        r = update();
        if (r != 0) r |= 0x80000000;
    }
    else
    {
        r = update();
        if (r != 0) r |= 0x40000000;
        save();
    }

    return r;
}

int app::Config::update()
{
    constexpr int ur_cfg_bit = 0x0001;
    constexpr int ur_sys_bit = 0x0002;
    // constexpr int ur_ls_bit = 0x0008;
    constexpr int ur_api_bit = 0x0004;
    updateResult = 0;

    try
    {
        std::string tmpStr = iniFile.getValueD(section_cfg, key_cfg_binVer, std::string(default_cfg_binVer));
        cfg_binVer = omw::Version(tmpStr);
        if (!cfg_binVer.isValid()) throw(-1);
    }
    catch (...)
    {
        cfg_binVer = omw::Version(std::string(default_cfg_binVer));
        updateResult |= ur_cfg_bit;
    }

    try
    {
        cfg_writeCfgFile = omw::stob(iniFile.getValueD(section_cfg, key_cfg_writeCfgFile, omw::to_string(default_cfg_writeCfgFile)));
    }
    catch (...)
    {
        // cfg_writeCfgFile = default_cfg_writeCfgFile;
        // updateResult |= ur_cfg_bit;
        //
        // special case

        cfg_writeCfgFile = true;
    }



    try
    {
        sys_machineId = iniFile.getValueD(section_sys, key_sys_machineId, std::string(default_sys_machineId));
    }
    catch (...)
    {
        sys_machineId = default_sys_machineId;
        updateResult |= ur_sys_bit;
    }

    try
    {
        sys_displayBrightness = std::stoi((iniFile.getValueD(section_sys, key_sys_displayBrightness, omw::to_string(default_sys_displayBrightness))));
    }
    catch (...)
    {
        sys_displayBrightness = default_sys_displayBrightness;
        updateResult |= ur_sys_bit;
    }

    try
    {
        sys_wifiCountry = iniFile.getValueD(section_sys, key_sys_wifiCountry, std::string(default_sys_wifiCountry));
    }
    catch (...)
    {
        sys_wifiCountry = default_sys_wifiCountry;
        updateResult |= ur_sys_bit;
    }



    // try
    //{
    //     ls_done = omw::stob(iniFile.getValueD(section_ls, key_ls_done, omw::to_string(default_ls_done)));
    // }
    // catch (...)
    //{
    //     ls_done = default_ls_done;
    //     updateResult |= ur_ls_bit;
    // }
    //
    // try
    //{
    //    ls_state = iniFile.getValueD(section_ls, key_ls_state, default_ls_state.data());
    //}
    // catch (...)
    //{
    //    ls_state = default_ls_state;
    //    updateResult |= ur_ls_bit;
    //}



    try
    {
        api_baseUrl = iniFile.getValueD(section_api, key_api_baseUrl, std::string(default_api_baseUrl));
    }
    catch (...)
    {
        api_baseUrl = default_api_baseUrl.data();
        updateResult |= ur_api_bit;
    }



    return updateResult;
}



app::Config app::config = app::Config();

``````


### src/application/appMain.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/appMain.h
`relative_path`: src/application/appMain.h
`format`: Arbitrary Binary Data
`size`: 773   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_APPMAIN_H
#define IG_APP_APPMAIN_H

#include <cstddef>
#include <cstdint>


enum EXITCODE // https://tldp.org/LDP/abs/html/exitcodes.html / on MSW are no preserved codes
{
    EC_OK = 0,
    EC_ERROR = 1,

    EC__begin_ = 79,

    EC_RPIHAL_INIT_ERROR = EC__begin_,
    // EC_..,

    EC__end_,

    EC__max_ = 113
};
static_assert(EC__end_ <= EC__max_, "too many error codes defined");
#if (defined(__unix__) || defined(__unix))
#include "sysexits.h"
static_assert(EC__begin_ == EX__MAX + 1, "maybe the first allowed user code has to be changed");
#endif // UNIX


namespace app {

int appMain();

} // namespace app


#endif // IG_APP_APPMAIN_H

``````


### src/application/appShared.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/appShared.h
`relative_path`: src/application/appShared.h
`format`: Arbitrary Binary Data
`size`: 460   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

/*

This is not shared between threads, it's only used in the main thread. So mutex and atomic is not needed.

*/

#ifndef IG_APP_APPSHARED_H
#define IG_APP_APPSHARED_H

#include <cstddef>
#include <cstdint>



namespace app {

/**
 * @brief Debounced mouse connection status.
 */
extern const bool& mouseConnected;

}


#endif // IG_APP_APPSHARED_H

``````


### src/application/localSetup.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/localSetup.h
`relative_path`: src/application/localSetup.h
`format`: Arbitrary Binary Data
`size`: 349   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_LOCALSETUP_H
#define IG_APP_LOCALSETUP_H

#include <cstddef>
#include <cstdint>


namespace app::localSetup {

enum
{
    running = 0,
    done,
    error,

    ___ret_end_
};

int task();

}


#endif // IG_APP_LOCALSETUP_H

``````


### src/application/dfu.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/dfu.h
`relative_path`: src/application/dfu.h
`format`: Arbitrary Binary Data
`size`: 331   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_DFU_H
#define IG_APP_DFU_H

#include <cstddef>
#include <cstdint>


namespace app::dfu {

enum
{
    running = 0,
    nop,
    done,
    failed,

    ___ret_end_
};

int task();

}


#endif // IG_APP_DFU_H

``````


### src/application/statusData.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/statusData.h
`relative_path`: src/application/statusData.h
`format`: Arbitrary Binary Data
`size`: 7668   




### src/application/treatment.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/application/treatment.h
`relative_path`: src/application/treatment.h
`format`: Arbitrary Binary Data
`size`: 532   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_TREATMENT_H
#define IG_APP_TREATMENT_H

#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

#include "application/treatmentData.h"



namespace app::treat {

void setPeltierIdle();
void setConfig(const app::treat::User& user, const app::treat::Config& cfg);

enum
{
    running = 0,
    done,

    ___ret_end_
};

int task();

} // namespace app::treat


#endif // IG_APP_TREATMENT_H

``````


### src/main.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/vacuul-machine-firmware/src/main.cpp
`relative_path`: src/main.cpp
`format`: Arbitrary Binary Data
`size`: 2575   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <iostream>
#include <string>
#include <vector>

#include "application/appMain.h"
#include "middleware/gpio.h"
#include "project.h"

#include <omw/cli.h>
#include <rpihal/rpihal.h>
#include <unistd.h>

#define LOG_MODULE_LEVEL LOG_LEVEL_DBG
#define LOG_MODULE_NAME  MAIN
#include "middleware/log.h"


namespace {}



int main(int argc, char** argv)
{
    int r = EC_OK;

#ifndef PRJ_DEBUG
    if (prj::version.isPreRelease()) { std::cout << omw::fgBrightMagenta << "pre-release v" << prj::version.toString() << omw::defaultForeColor << std::endl; }
#endif

    RPIHAL___setModel___(RPIHAL_model_4B); // temporary hack!       TODO update rpihal
#ifdef RPIHAL_EMU
    if (RPIHAL_EMU_init(RPIHAL_model_4B) == 0)
    {
        while (!RPIHAL_EMU_isRunning()) {}
    }
#endif // RPIHAL_EMU

    if (GPIO_init() == 0)
    {
#if defined(PRJ_DEBUG) && 1
        gpio::dispBacklightEn->write(0);
#endif

        // disable peltier power and pump
        gpio::peltPowerL->write(0);
        gpio::peltPowerR->write(0);
        gpio::pumpA->write(0);
        gpio::pumpB->write(0);
    }
    else { r = EC_RPIHAL_INIT_ERROR; }

#if defined(PRJ_DEBUG) && 0
    RPIHAL_GPIO_dumpAltFuncReg(0x3c0000);
    RPIHAL_GPIO_dumpPullUpDnReg(0x3c0000);
#endif

#if (LOG_MODULE_LEVEL >= LOG_LEVEL_INF)
    {
        const char* dbgStr = "";
#ifdef PRJ_DEBUG
        dbgStr = LOG_SGR_BMAGENTA " DEBUG";
#endif
        LOG_INF(LOG_SGR_BYELLOW "%s " LOG_SGR_BCYAN "v%s%s", prj::appName, prj::version.toString().c_str(), dbgStr);
    }
#endif

    LOG_DBG("pid: %i", (int)getpid());

    while (r == EC_OK)
    {
        gpio::task();

        r = app::appMain();

        usleep(5 * 1000);

#ifdef RPIHAL_EMU
        if (!RPIHAL_EMU_isRunning())
        {
            LOG_WRN("rpihal emu terminate");
            r = EC_OK;
            break;
        }
#endif // RPIHAL_EMU
    }

    // TODO is this OK? doesn't the hw make strage stuff if outputs are hi-Z and have pull up/down?
    // GPIO_reset();
    // or define a fallback config
    gpio::led::run->clr();

#ifdef RPIHAL_EMU
    RPIHAL_EMU_cleanup();

    // wait to prevent segmentation fault
    // util::sleep(10000); doesn't help
#endif // RPIHAL_EMU

    return r;
}



#if PRJ_USE_HTML_IO_EMU && !defined(PRJ_DEBUG)
#error "PRJ_USE_HTML_IO_EMU is enabled in release build"
#endif

#if PRJ_PKG_DEV_DISPDMY && !PRJ_USE_HTML_IO_EMU
#error "vacuul-dev-dispdmy needs HTML io emu"
#endif

``````



## DemoRecurrentV1
`clone_url`: https://github.com/vacuul-dev/DemoRecurrentV1.git


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/DemoRecurrentV1/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 986   


``````
# ⚡ Deno Starter Function

A simple starter function. Edit `src/main.ts` to get started and create something awesome! 🚀

## 🧰 Usage

### GET /ping

- Returns a "Pong" message.

**Response**

Sample `200` Response:

```text
Pong
```

### GET, POST, PUT, PATCH, DELETE /

- Returns a "Learn More" JSON response.

**Response**

Sample `200` Response:

```json
{
  "motto": "Build like a team of hundreds_",
  "learn": "https://appwrite.io/docs",
  "connect": "https://appwrite.io/discord",
  "getInspired": "https://builtwith.appwrite.io"
}
```

## ⚙️ Configuration

| Setting           | Value                    |
| ----------------- | ------------------------ |
| Runtime           | Deno (1.35)              |
| Entrypoint        | `src/main.ts`            |
| Build Commands    | `deno cache src/main.ts` |
| Permissions       | `any`                    |
| Timeout (Seconds) | 15                       |

## 🔒 Environment Variables

No environment variables required.

``````


### src/main.ts
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/DemoRecurrentV1/src/main.ts
`relative_path`: src/main.ts
`format`: Arbitrary Binary Data
`size`: 1284   


``````
import { Client, Users } from "https://deno.land/x/appwrite@7.0.0/mod.ts";

// This Appwrite function will be executed every time your function is triggered
export default async ({ req, res, log, error }: any) => {
  // You can use the Appwrite SDK to interact with other services
  // For this example, we're using the Users service
  const client = new Client()
    .setEndpoint(Deno.env.get("APPWRITE_FUNCTION_API_ENDPOINT") ?? '')
    .setProject(Deno.env.get("APPWRITE_FUNCTION_PROJECT_ID") ?? '')
    .setKey(req.headers['x-appwrite-key'] ?? '');
  const users = new Users(client);

  try {
    const response = await users.list();
    // Log messages and errors to the Appwrite Console
    // These logs won't be seen by your end users
    log(`Total users: ${response.total}`);
  } catch(err) {
    error("Could not list users: " + err.message);
  }

  // The req object contains the request data
  if (req.path === "/ping") {
    // Use res object to respond with text(), json(), or binary()
    // Don't forget to return a response!
    return res.text("Pong");
  }

  return res.json({
    motto: "Build like a team of hundreds_",
    learn: "https://appwrite.io/docs",
    connect: "https://appwrite.io/discord",
    getInspired: "https://builtwith.appwrite.io",
  });
};

``````



## machine-firmware
`clone_url`: https://github.com/vacuul-dev/machine-firmware.git


### tools/display-mockup/deploy.pjob
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/display-mockup/deploy.pjob
`relative_path`: tools/display-mockup/deploy.pjob
`format`: Arbitrary Binary Data
`size`: 690   


``````
#
# author        Oliver Blaser
# copyright     Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
#


# This is a potoroo job file. Use v0.2.0 (https://github.com/oblaser/potoroo/releases/tag/v0.2.0) for processing this job file.


-if ./src/index.html                -od ./deploy/www/   -Werror     -t custom:<!--#p    -Wsup 107
-if ./src/style.css                 -od ./deploy/www/   -Werror     -t custom:/*#p      -Wsup 107
-if ./src/index.js                  -od ./deploy/www/   -Werror
-if ./src/jquery-3.6.0.min.js       -od ./deploy/www/   --copy
-if ./src/lib.js                    -od ./deploy/www/   -Werror
-if ./src/rpi-wifi-country.js       -od ./deploy/www/   -Werror

``````


### tools/display-mockup/src/index.html
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/display-mockup/src/index.html
`relative_path`: tools/display-mockup/src/index.html
`format`: Arbitrary Binary Data
`size`: 1981   


``````
<!--
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
-->

<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=yes">

    <title>Vacuul Mockup</title>

    <link type="text/css" rel="stylesheet" href="./style.css">

    <!--#p rm -->
    <script>
        const websocket_url = 'ws://192.168.188.60:1105';
        //const websocket_url = 'ws://10.0.0.11:1105';
        //const websocket_url = 'ws://' + window.location.host + ':1105';
    </script>
    <!--#p endrm -->
    <!--#p ins <script>const websocket_url = 'ws://localhost:1105'; /* 127.0.0.1 does not work with js! */ </script> <!-- -->

    <script src="./jquery-3.6.0.min.js"></script>
    <script src="./lib.js?cachebuster=2"></script>
    <script src="./rpi-wifi-country.js"></script>
    <script src="./index.js?cachebuster=11"></script>

</head>
<body>

    <noscript>
        <div>
            <div style="display: inline-block; border-radius: 5px; border: solid 2px var(--errorFg); margin-bottom: 8px; padding: 9px 13px; background-color: var(--errorBg); color: var(--errorFg); font-weight: normal; font-size: 16px; font-family: Helvetica, sans-serif;">
                This site uses JavaScript. Please enable JavaScript to view the site correctly.
            </div>
        </div>
    </noscript>

    <div class="mainContainer">

        <div class="pageTitle">Vacuul Functional Display Mockup</div>

        <div id="display-container" style="min-height: 450px;">
            <div style="min-height: 100px; background-color: firebrick; font-weight: bold; font-size: 30px; color: whitesmoke; font-style: italic;">
                waiting for software to boot...
            </div>
        </div>

        <!--#p rm -->
        <div class="pageTitle2">Log</div>
        <div id="log-container"></div>
        <!--#p endrm -->
    </div>

</body>
</html>

``````


### tools/display-mockup/src/jquery-3.6.0.min.js
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/display-mockup/src/jquery-3.6.0.min.js
`relative_path`: tools/display-mockup/src/jquery-3.6.0.min.js
`format`: Arbitrary Binary Data
`size`: 89501   




### tools/display-mockup/src/lib.js
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/display-mockup/src/lib.js
`relative_path`: tools/display-mockup/src/lib.js
`format`: Arbitrary Binary Data
`size`: 2348   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/



function deep_clone(v) { return JSON.parse(JSON.stringify(v)); }

// returns a random integer in range [0, end)
function randInt(end = 2147483647) { return Math.floor(Math.random() * end); }

function str_pad2_ui(v) { return (v < 10 ? '0' + v : v); }

Date.prototype.myString = function() {
    let str = '';

    str += str_pad2_ui(this.getDate()) + '.' + str_pad2_ui(this.getMonth() + 1) + '.' + this.getFullYear();
    str += ' ';
    str += str_pad2_ui(this.getHours()) + ':' + str_pad2_ui(this.getMinutes()) + ':' + str_pad2_ui(this.getSeconds());

    return str;
};

Date.prototype.myIsoString = function() {
    let str = '';

    str += this.getFullYear() + '-' + str_pad2_ui(this.getMonth() + 1) + '-' + str_pad2_ui(this.getDate());
    str += 'T';
    str += str_pad2_ui(this.getHours()) + ':' + str_pad2_ui(this.getMinutes()) + ':' + str_pad2_ui(this.getSeconds());

    // ISO 8601 zone offset = localTime - gmtTime
    // z [min] = gmtTime - localTime
    let z = this.getTimezoneOffset(); // returns the local offset, what ever this Date object is representing

    if (z > 0) { str += '-'; }
    else { str += '+'; }
    z = Math.abs(z);

    str += str_pad2_ui(Math.floor(z / 60));
    str += str_pad2_ui(z % 60);

    return str;
};

function escapeHtml(str) { return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#039;"); }

function jsonToHtml(obj, indent = 2)
{
    let html = '';

    html += '<pre>';
    html += escapeHtml(JSON.stringify(obj, null, indent));
    html += '</pre>';

    return html;
}

function ojs_roundSignificant(value, nDigits = 4)
{
    let r = 0;

    if (value != 0)
    {
        if (value === Infinity) r = Infinity;
        else if (value === -Infinity) r = -Infinity;
        else
        {
            let signfactor = (value > 0 ? 1 : -1);
            value *= signfactor;

            let range = Math.floor(Math.log10(value));

            if (range >= (nDigits - 1)) r = Math.round(value);
            else
            {
                let fact = Math.pow(10, (nDigits - range - 1));
                r = Math.round(value * fact) / fact;
            }

            r *= signfactor;
        }
    }

    return r;
}

``````


### tools/display-mockup/src/index.js
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/display-mockup/src/index.js
`relative_path`: tools/display-mockup/src/index.js
`format`: Arbitrary Binary Data
`size`: 31365   




### tools/display-mockup/src/style.css
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/display-mockup/src/style.css
`relative_path`: tools/display-mockup/src/style.css
`format`: Arbitrary Binary Data
`size`: 2726   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

:root
{
    --color1: #d3d3d3;
    --color2: #b3b3b3;
    --color3: #5c5c5c;
    --colorText: #424242;
    --colorHighlight: #4b84ff;

    --errorFg: #ffd900;
    --errorBg: #424242;

    --fontFamily: "Trebuchet MS", Helvetica, sans-serif;
}


body
{
    background-color: var(--color2);

    color: var(--colorText);

    font-family: var(--fontFamily);
    font-size: 16px;
}

a { color: var(--colorHighlight); }
a:hover { color: inherit; }

pre
{
    display: block;
    margin: 0px;
    font-family: "Courier New", monospace;
}

.mainContainer
{
    margin-left: auto;
    margin-right: auto;
    width: 98%;
    /*max-width: 820px;/**/

    /*border-top: rgb(0, 186, 233) 10px solid; /* uncomment in dev */
    /*border-right: rgb(0, 186, 233) 10px solid; /* uncomment in dev */
}

.pageTitle
{
    font-size: larger;
    font-weight: bold;
    margin-top: 0px;
    margin-bottom: 7px;
}

.pageTitle2
{
    font-size: large;
    margin-top: 3px;
    margin-bottom: 3px;
}
/*#p rm
~
    https://stackoverflow.com/questions/2717480/css-selector-for-first-element-with-class/8539107#8539107
    https://stackoverflow.com/questions/3615518/css-selector-to-select-first-element-of-a-given-class/3615559#3615559
+
    https://stackoverflow.com/questions/59344276/css-selector-first-element-with-class-in-a-group-of-same-class
/*#p endrm */
.pageTitle2 ~ .pageTitle2 /* selecting all but the first */
{
    margin-top: 30px;
}

.pageTitle3
{
    font-weight: bold;
    margin-top: 0px;
    margin-bottom: 3px;
}
.pageTitle3 ~ .pageTitle3 /* selecting all but the first */
{
    margin-top: 10px;
}



.localSetupBox
{
    width: 50%;
    padding: 20px 15px;
    margin: 0px auto;
    margin-bottom: 30px;
    background-color: aquamarine;
    border-radius: 10px;
}

.localSetupBoxTitle
{
    text-align: center;
    font-size: 20px;
}

.errorBox
{
    display: inline-block;
    margin: 10px 0px 0px 10px;
    padding: 5px 10px;
    text-align: center;
    background-color: #ff0080;
}

.formAnsBox
{
    display: inline-block;
    margin-left: 20px;
}

.formButton
{
    display: inline-block;
    margin: 2px;
    padding: 1px 2px;
    text-align: center;
    border: 1px var(--colorText) solid;
    width: 75px;
    background-color: var(--color2);
}

.formButton:hover
{
    border: 1px var(--colorHighlight) solid;
    color: var(--colorHighlight);
}

.kbBtn
{
    display: inline-block;
    margin: 2px;
    padding: 1px 2px;
    min-width: 19px;
    text-align: center;
    border: 1px var(--colorText) solid;
}

.kbBtn:hover
{
    border: 1px var(--colorHighlight) solid;
    color: var(--colorHighlight);
}

``````


### tools/display-mockup/src/rpi-wifi-country.js
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/display-mockup/src/rpi-wifi-country.js
`relative_path`: tools/display-mockup/src/rpi-wifi-country.js
`format`: Arbitrary Binary Data
`size`: 10483   




### tools/html-io-emu/deploy.pjob
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/html-io-emu/deploy.pjob
`relative_path`: tools/html-io-emu/deploy.pjob
`format`: Arbitrary Binary Data
`size`: 742   


``````
#
# author        Oliver Blaser
# copyright     Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
#


# This is a potoroo job file. Use v0.2.0 (https://github.com/oblaser/potoroo/releases/tag/v0.2.0) for processing this job file.


-if ./src/index.html    -od ./deploy/intermediate/      -Werror     -t custom:<!--#p    -Wsup 107
-if ./src/style.css     -od ./deploy/intermediate/      -Werror     -t custom:/*#p      -Wsup 107
-if ./src/index.js      -od ./deploy/intermediate/      -Werror
-if ./src/lib.js        -od ./deploy/intermediate/      -Werror

-if ./deploy/intermediate/index.html    -od ./deploy/www/   -Werror     -t custom:/*#p      -Wsup 107

-if ../display-mockup/src/jquery-3.6.0.min.js   -od ./deploy/www/   --copy

``````


### tools/html-io-emu/src/index.html
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/html-io-emu/src/index.html
`relative_path`: tools/html-io-emu/src/index.html
`format`: Arbitrary Binary Data
`size`: 2295   


``````
<!--
author          Oliver Blaser
date            05.09.2024
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
-->

<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=yes">

    <title>Vacuul IO Emu</title>

    <!--#p rmn 1 -->
    <link type="text/css" rel="stylesheet" href="./style.css">
    <style>
        /*#p include 'style.css' */
    </style>

    <!--#p rm -->
    <script>
        console.log(window.location.host);
        const websocket_url = 'ws://192.168.188.60:1105';
        //const websocket_url = 'ws://192.168.188.64:1105';
    </script>
    <!--#p endrm -->
    <!--#p ins <script>const websocket_url = 'ws://' + window.location.host + ':1105'; </script> <!-- -->

    <!--#p ins <script src="./jquery-3.6.0.min.js"></script> <!-- -->

    <!--#p rm -->
    <script src="https://code.jquery.com/jquery-3.6.0.min.js" integrity="sha256-/xUj+3OJU5yExlq6GSYGSHk7tPXikynS7ogEvDej/m4=" crossorigin="anonymous"></script>
    <script src="./lib.js?cachebuster=1"></script>
    <script src="./index.js?cachebuster=1"></script>
    <!--#p endrm -->

    <script>
        /*#p include 'lib.js' */
        /*#p include 'index.js' */
    </script>

</head>
<body>

    <noscript>
        <div>
            <div style="display: inline-block; border-radius: 5px; border: solid 2px var(--errorFg); margin-bottom: 8px; padding: 9px 13px; background-color: var(--errorBg); color: var(--errorFg); font-weight: normal; font-size: 16px; font-family: Helvetica, sans-serif;">
                This site uses JavaScript. Please enable JavaScript to view the site correctly.
            </div>
        </div>
    </noscript>

    <div class="mainContainer">

        <div class="pageTitle">Vacuul IO Emulator</div>

        <div id="display-container" style="min-height: 450px;">
            <div style="min-height: 100px; background-color: firebrick; font-weight: bold; font-size: 30px; color: whitesmoke; font-style: italic;">
                waiting for software to boot...
            </div>
        </div>

        <!--#p rm -->
        <div class="pageTitle2">Log</div>
        <div id="log-container"></div>
        <!--#p endrm -->
    </div>

</body>
</html>

``````


### tools/html-io-emu/src/lib.js
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/html-io-emu/src/lib.js
`relative_path`: tools/html-io-emu/src/lib.js
`format`: Arbitrary Binary Data
`size`: 1702   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/



function deep_clone(v) { return JSON.parse(JSON.stringify(v)); }

// returns a random integer in range [0, end)
function randInt(end = 2147483647) { return Math.floor(Math.random() * end); }

function str_pad2_ui(v) { return (v < 10 ? '0' + v : v); }

Date.prototype.myString = function() {
    let str = '';

    str += str_pad2_ui(this.getDate()) + '.' + str_pad2_ui(this.getMonth() + 1) + '.' + this.getFullYear();
    str += ' ';
    str += str_pad2_ui(this.getHours()) + ':' + str_pad2_ui(this.getMinutes()) + ':' + str_pad2_ui(this.getSeconds());

    return str;
};

Date.prototype.myIsoString = function() {
    let str = '';

    str += this.getFullYear() + '-' + str_pad2_ui(this.getMonth() + 1) + '-' + str_pad2_ui(this.getDate());
    str += 'T';
    str += str_pad2_ui(this.getHours()) + ':' + str_pad2_ui(this.getMinutes()) + ':' + str_pad2_ui(this.getSeconds());

    // ISO 8601 zone offset = localTime - gmtTime
    // z [min] = gmtTime - localTime
    let z = this.getTimezoneOffset(); // returns the local offset, what ever this Date object is representing

    if (z > 0) { str += '-'; }
    else { str += '+'; }
    z = Math.abs(z);

    str += str_pad2_ui(Math.floor(z / 60));
    str += str_pad2_ui(z % 60);

    return str;
};

function escapeHtml(str) { return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#039;"); }

function jsonToHtml(obj, indent = 2)
{
    let html = '';

    html += '<pre>';
    html += escapeHtml(JSON.stringify(obj, null, indent));
    html += '</pre>';

    return html;
}

``````


### tools/html-io-emu/src/index.js
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/html-io-emu/src/index.js
`relative_path`: tools/html-io-emu/src/index.js
`format`: Arbitrary Binary Data
`size`: 8951   




### tools/html-io-emu/src/style.css
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/html-io-emu/src/style.css
`relative_path`: tools/html-io-emu/src/style.css
`format`: Arbitrary Binary Data
`size`: 3825   


``````
/*
author          Oliver Blaser
date            05.09.2024
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

:root
{
    --color1: #d3d3d3;
    --color2: #b3b3b3;
    --color3: #5c5c5c;
    --colorText: #424242;
    --colorHighlight: #4b84ff;

    --errorFg: #ffd900;
    --errorBg: #424242;

    --fontFamily: "Trebuchet MS", Helvetica, sans-serif;
}


body
{
    background-color: var(--color2);

    color: var(--colorText);

    font-family: var(--fontFamily);
    font-size: 16px;
}

a { color: var(--colorHighlight); }
a:hover { color: inherit; }

pre
{
    display: block;
    margin: 0px;
    font-family: "Courier New", monospace;
}

.mainContainer
{
    margin-left: auto;
    margin-right: auto;
    width: 98%;
    /*max-width: 820px;/**/

    /*border-top: rgb(0, 186, 233) 10px solid; /* uncomment in dev */
    /*border-right: rgb(0, 186, 233) 10px solid; /* uncomment in dev */
}

.pageTitle
{
    font-size: larger;
    font-weight: bold;
    margin-top: 0px;
    margin-bottom: 7px;
}

.pageTitle2
{
    font-size: large;
    margin-top: 3px;
    margin-bottom: 3px;
}
/*#p rm
~
    https://stackoverflow.com/questions/2717480/css-selector-for-first-element-with-class/8539107#8539107
    https://stackoverflow.com/questions/3615518/css-selector-to-select-first-element-of-a-given-class/3615559#3615559
+
    https://stackoverflow.com/questions/59344276/css-selector-first-element-with-class-in-a-group-of-same-class
/*#p endrm */
.pageTitle2 ~ .pageTitle2 /* selecting all but the first */
{
    margin-top: 30px;
}

.pageTitle3
{
    font-weight: bold;
    margin-top: 0px;
    margin-bottom: 3px;
}
.pageTitle3 ~ .pageTitle3 /* selecting all but the first */
{
    margin-top: 10px;
}



.localSetupBox
{
    width: 50%;
    padding: 20px 15px;
    margin: 0px auto;
    margin-bottom: 30px;
    background-color: aquamarine;
    border-radius: 10px;
}

.localSetupBoxTitle
{
    text-align: center;
    font-size: 20px;
}

.errorBox
{
    display: inline-block;
    margin: 10px 0px 0px 10px;
    padding: 5px 10px;
    text-align: center;
    background-color: #ff0080;
}

.formAnsBox
{
    display: inline-block;
    margin-left: 20px;
}

.formButton
{
    display: inline-block;
    margin: 2px;
    padding: 1px 10px;
    text-align: center;
    border: 1px var(--colorText) solid;
    width: 75px;
    background-color: var(--color2);
}

.formButton:hover
{
    border: 1px var(--colorHighlight) solid;
    color: var(--colorHighlight);
}

.kbBtn
{
    display: inline-block;
    margin: 2px;
    padding: 1px 2px;
    min-width: 19px;
    text-align: center;
    border: 1px var(--colorText) solid;
}

.kbBtn:hover
{
    border: 1px var(--colorHighlight) solid;
    color: var(--colorHighlight);
}



.formToggleSwitch
{
    position: relative;
    display: inline-block;
    width: 60px;
    height: 34px;
}

.formToggleSwitch input
{
    opacity: 0;
    width: 0;
    height: 0;
}

.formToggleSwitchSlider
{
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: #ccc;
    -webkit-transition: .4s;
    transition: .4s;
}

.formToggleSwitchSlider:before
{
    position: absolute;
    content: "";
    height: 26px;
    width: 26px;
    left: 4px;
    bottom: 4px;
    background-color: white;
    -webkit-transition: .4s;
    transition: .4s;
}

input:checked+.formToggleSwitchSlider
{
    background-color: #2196F3;
}

input:focus+.formToggleSwitchSlider
{
    box-shadow: 0 0 1px #2196F3;
}

input:checked+.formToggleSwitchSlider:before
{
    -webkit-transform: translateX(26px);
    -ms-transform: translateX(26px);
    transform: translateX(26px);
}

/* Rounded sliders */
.formToggleSwitchSlider.round
{
    border-radius: 34px;
}

.formToggleSwitchSlider.round:before
{
    border-radius: 50%;
}

``````


### tools/colors/treatment.m
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/colors/treatment.m
`relative_path`: tools/colors/treatment.m
`format`: Arbitrary Binary Data
`size`: 624   


``````
%
% author        Oliver Blaser
% copyright     Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
%

clf
clear -a
%clc
format short eng



n = 100;

relHue = sin(linspace(0, pi, n)) .^ 2;

startHue = 10;
stopHue = 35;

hue = (startHue + relHue .* (stopHue-startHue)) ./ 360;
round(hue' .* 360)


hsvTable = [hue', ones(n,1), ones(n,1)];

% is "normal" RGB table, not calculated for hardware PWM LED output
table = hsv2rgb(hsvTable);

ui8table = round(table .* 255);


subplot(4,1,[1:2])
hold on
grid on
plot([1:1:n],relHue)
plot([1:1:n],hue)

subplot(4,1,3)
rgbplot(table, "composite")

subplot(4,1,4)
rgbplot(table)

``````


### tools/colors/commission.m
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/tools/colors/commission.m
`relative_path`: tools/colors/commission.m
`format`: Arbitrary Binary Data
`size`: 952   


``````
%
% author        Oliver Blaser
% copyright     Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
%

clf
clear -a
%clc
format short eng



function writeCFile(table)

  type = 'uint8_t';

  tableSize = size(table, 1);

  fd = fopen(['commission.c'], 'w+');
  fprintf(fd, '\nstatic %s table[%i] = {\n    ', type, tableSize);

  for i=1:tableSize
    fprintf(fd, '    { %3i, %3i, %3i },\n', table(i,1), table(i,2), table(i,3));
  end

  fprintf(fd, '\n};\n');
  fclose(fd);

endfunction



n = 50;

relHue = sin(linspace(0, pi/2, n)) .^ 10;
hue = (240 + relHue .* (360-240)) ./ 360;
round(hue' .* 360)


hsvTable = [hue', ones(n,1), ones(n,1)];

% is "normal" RGB table, not calculated for hardware PWM LED output
table = hsv2rgb(hsvTable);

ui8table = round(table .* 255);
writeCFile(ui8table);


subplot(4,1,[1:2])
hold on
grid on
plot([1:1:n],relHue)
plot([1:1:n],hue)

subplot(4,1,3)
rgbplot(table, "composite")

subplot(4,1,4)
rgbplot(table)

``````


### release-notes.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/release-notes.md
`relative_path`: release-notes.md
`format`: Arbitrary Binary Data
`size`: 2809   


``````
# Vacuul Release Notes
Version Numbers of Debian packages.



### v0.3.2-1 - YYYY-MM-DD
- 



### v0.3.1-1 - 2025-02-03
- Fixed test sound for sudo exec



### v0.3.0-1 - 2025-01-31
- Updated to API v1.2, still not using it
- Added hardware version to status data
- Play test sound on entering commission state
- Display WebApp 2025-01-31_14-31
- Back to normal treatment (see releases [v0.2.11-1](#v0211-1---2025-01-10) and [v0.2.10-1](#v0210-1---2025-01-09))



### v0.2.11-1 - 2025-01-10
> This release is not added to the DFU index.
- Special treatment settings (6 blocks, (idle 16°C) 15°C to 10°C, block duration 2min, 10s pause)



### v0.2.10-1 - 2025-01-09
> This release is not added to the DFU index.
- Special treatment settings (6 blocks, (idle 16°C) 15°C to 10°C, block duration 2min, no pause)



### v0.2.9-1 - 2024-11-28
- System setup: Fixed URL in chromium autostart
- Fixed URL when not using API
- Using 3 point controller
- Display WebApp 2024-11-28_12-20



### v0.2.8-1 - 2024-11-28
> **Caution!** This release can't be used, major bug with URL when not using API (which is the case in this release).

- MAX32664 removed DFU (was disabled) from source
- Save WiFi country in config file
- Simple DFU from `static.blaser-eng.ch/vacuul/dfu/v1/`
- System setup
    - Disabled chromium cache
    - Specified chromium user data dir to be on `/tmp`
    - IP Ports
- Get API host from config file
- Display enable by WS connection
- Biom data timeout only if hands are on
- Display WebApp 2024-11-28_12-20



### v0.2.7-1 - 2024-11-07
- Removed config.ini from debian package
- Emulate pelt temp for dispdmy
- Biometric null data timeout individual per value
- MAX32664 disabled data confidence filter
- Display WebApp 2024-11-07_13-55



### v0.2.6-1 - 2024-11-04
- Biometric null data timeout 15s
- Treatment Pause 2min



### v0.2.5-1 - 2024-11-04
- Biometric data is written to 0 after 10s null data
- Display WebApp v1.0.0



### v0.2.4-1 - 2024-11-04
- Local setup config WiFi country
- Don't reset MAX32664 on neither treatment start nor end
- Display backlight control signals
- MAX32664
    - _DFU_ (disabled atm)
    - Never read algorithm version
    - Fixed sample size depending on hub version
- MAX32664A
    - Analyze all received samples
    - Allow v1.9.x
- Fans A (10°C) B C temp dependent
- Hands-off debounce 1s



### v0.2.3
Not an actual release, inc version for updateable deb package (see commit 1730420 2024-10-10 11:45)



### v0.2.2-3 - 2024-10-03
- 20s delay dual to single pump
- Debounce for hands off
- MAX32664 booted state doesn't matter in I2C thread



### v0.2.0-1 - 2024-10-03
- Peltier hold start temperature in idle
- Peltier temperature in pause block
- Fans allways on
- I2C thread is self healing
- Don't use API / treatment starts on hands on

``````


### dfu/index.jsonc
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/dfu/index.jsonc
`relative_path`: dfu/index.jsonc
`format`: Arbitrary Binary Data
`size`: 478   


``````
{
    "interfaceVersion" : "1.0.0",
    "packages" :
    [
        { "upstreamVersion" : "0.2.9", "debianRevision" : 1, "file" : "vacuul_0.2.9-1.deb" },
        // vacuul_0.2.10+SPECIALTREATMENT-1.deb is not indexed
        // vacuul_0.2.11+SPECIALTREATMENT-1.deb is not indexed
        // vacuul_0.3.0-1.deb is not indexed
        { "upstreamVersion" : "0.3.1", "debianRevision" : 1, "file" : "vacuul_0.3.1-1.deb" },

        {} // dummy last element, could be omitted
    ]
}

``````


### release-notes_pre-releases.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/release-notes_pre-releases.md
`relative_path`: release-notes_pre-releases.md
`format`: Arbitrary Binary Data
`size`: 3396   


``````
# Vacuul Pre-Release Notes
Version Numbers of Debian packages.

Release notes for test softwares and pre-releases.



### [v0.3.2-1 - YYYY-MM-DD](./release-notes.md#v032-1---yyyy-mm-dd)

### [v0.3.1-1 - 2025-02-03](./release-notes.md#v031-1---2025-02-03)

### [v0.3.0-1 - 2025-01-31](./release-notes.md#v030-1---2025-01-31)

### v0.3.0~alpha-1
- Updated to API v1.1, and using API
- Added connection flags in IO emu



------------------------------------------------------------------------------------------------------------------------

### [v0.2.11-1 - 2025-01-10](./release-notes.md#v0211-1---2025-01-10)

### [v0.2.10-1 - 2025-01-09](./release-notes.md#v0210-1---2025-01-09)

### [v0.2.9-1 - 2024-11-28](./release-notes.md#v029-1---2024-11-28)

### [v0.2.8-1 - 2024-11-28](./release-notes.md#v028-1---2024-11-28)

### v0.2.8~alpha.2-1
- IO emu generates more error messages
- `vacuul-dev-dispdmy` release
    - Normal treatments
    - No API, treatment starts by IO emu or hands on
    - 30s boot time



### v0.2.8~alpha.1-1
- Get API host from config file
- `vacuul-dev-dispdmy`
    - Short dev treatments
    - Using API



### v0.2.8~aaaaa.dfuTest.14
- Display WebApp  `034_vacuul-display-webapp_v0.2.5.zip` (only for this package)



### v0.2.8~aaaaa.dfuTest.10 ... .13
- Simple DFU from `static.blaser-eng.ch/vacuul/dfu/v1/`



### v0.2.8~aaaaa.dfuTest.1 ... .3
- DFU test packages (testing `apt-get install ./vacuul*.deb` while running)



------------------------------------------------------------------------------------------------------------------------

### [v0.2.7-1 - 2024-11-07](./release-notes.md#v027-1---2024-11-07)

### v0.2.7~beta-1
- Biometric null data timeout individual per value
- MAX32664 disabled data confidence filter
- Display WebApp 2024-11-05_20-01



### v0.2.7~alpha-1
- Emulate pelt temp for dispdmy
- `vacuul-dev-dispdmy` release with short dev treatments



------------------------------------------------------------------------------------------------------------------------

### [v0.2.6-1 - 2024-11-04](./release-notes.md#v026-1---2024-11-04)

### [v0.2.5-1 - 2024-11-04](./release-notes.md#v025-1---2024-11-04)

### [v0.2.4-1 - 2024-11-04](./release-notes.md#v024-1---2024-11-04)

### v0.2.4~alpha.4-1
- `vacuul-dev-dispdmy` release with short dev treatments



### v0.2.4~alpha.3-1
- Undone "DEBUG MAX32664*" from [v0.2.4~alpha.1-1](#v024alpha1-1)
- _MAX32664 DFU_ (disabled atm)
- Some fixes on IO emu
- `vacuul-dev-dispdmy` release with normal treatments



### v0.2.4~alpha.2-1
- MAX32664 never read algorithm version



### v0.2.4~alpha.1-1
- Local setup config country
- MAX32664A analyze all received samples
- MAX32664 fixed sample size depending on hub version
- DEBUG MAX32664A no min SWV
- DEBUG MAX32664 log raw sample data



### v0.2.4~alpha-1
- Don't reset MAX32664 on neither treatment start nor end
- Display backlight control signals



------------------------------------------------------------------------------------------------------------------------

### [v0.2.3](./release-notes.md#v023)

### v0.2.3-alpha-1
- Enabled log output to HW debug MAX32664 I2C



------------------------------------------------------------------------------------------------------------------------

### [v0.2.2-3 - 2024-10-03](./release-notes.md#v022-3---2024-10-03)

### [v0.2.0-1 - 2024-10-03](./release-notes.md#v020-1---2024-10-03)

``````


### readme.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/readme.md
`relative_path`: readme.md
`format`: Arbitrary Binary Data
`size`: 458   


``````
# Vacuul

help:
- [doc](./doc/)
- [API Specification](https://github.com/vacuul-dev/api-specification)



---

## TODOs
### DFU Process
There was an idea to create a separate screen for the DFU process with an extended log on the display. Thats why the DFU process has the same software structure as the other main tasks.
At the moment it is run inside the local setup mode, and minimalistic status info is displayed. A final decision has not been made yet.

``````


### sdk/json/include/json/json.hpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/sdk/json/include/json/json.hpp
`relative_path`: sdk/json/include/json/json.hpp
`format`: Arbitrary Binary Data
`size`: 919975   




### doc/paths.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/doc/paths.md
`relative_path`: doc/paths.md
`format`: Arbitrary Binary Data
`size`: 484   


``````
# Debian Packages

## vacuul (release)
```txt
/etc/opt/vacuul
/etc/vacuul-otp.ini
/opt/vacuul
/var/www/html/display           Display Mockup
/var/www/html/                  Display Release
```

## vacuul-dev
```txt
/etc/opt/vacuul-dev
/etc/vacuul-otp.ini
/opt/vacuul-dev
/var/www/html/dev-mockup
/var/www/html/dev-mockup-dbg
/var/www/html/                  Display Release
```

## vacuul-dev-dispdmy
```txt
/etc/opt/vacuul-dev-dispdmy
/opt/vacuul-dev-dispdmy
/var/www/html/io-emu
```

``````


### doc/raspi.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/doc/raspi.md
`relative_path`: doc/raspi.md
`format`: Arbitrary Binary Data
`size`: 6716   




### build/dep_vstr.txt
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/build/dep_vstr.txt
`relative_path`: build/dep_vstr.txt
`format`: Arbitrary Binary Data
`size`: 12   


``````
0.3.2-alpha

``````


### build/pack_deb_dev-dispdmy.sh
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/build/pack_deb_dev-dispdmy.sh
`relative_path`: build/pack_deb_dev-dispdmy.sh
`format`: Shell Script
`size`: 1935   




### build/cmake/CMakeLists.txt
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/build/cmake/CMakeLists.txt
`relative_path`: build/cmake/CMakeLists.txt
`format`: Arbitrary Binary Data
`size`: 3654   


``````

# author        Oliver Blaser
# copyright     Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG

cmake_minimum_required(VERSION 3.13)

project(vacuul)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED true)

# detect if target platform is RasPi (actually any ARM platform, may be improved)
set(PLAT_IS_RASPI false)
#message(${CMAKE_SYSTEM_PROCESSOR})
if(CMAKE_SYSTEM_PROCESSOR MATCHES "(armv[6-8]([A-Z]|[a-z])?)" OR CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
    set(PLAT_IS_RASPI true)
endif()



#
# SDK
#

# omw
add_subdirectory(${CMAKE_CURRENT_LIST_DIR}/../../sdk/omw/build/cmake/libomw.a/ ${CMAKE_CURRENT_LIST_DIR}/../../sdk/omw/build/cmake/libomw.a/)
include_directories(../../sdk/omw/include)

# WebSocket++
#   force WebSocket++ to use 100% C++11 mode, so that it doesn't try to look for Boost (Note that under MinGW,
#   Boost.Thread is still required, due to a bug in MinGW that prevents the C++11 version from being used).
add_definitions(-D_WEBSOCKETPP_CPP11_STRICT_)
include_directories(../../sdk/websocketpp)

# ASIO
add_definitions(-DASIO_SEPARATE_COMPILATION -DASIO_STANDALONE)
include_directories(../../sdk/asio/asio/include)

# json
include_directories(../../sdk/json/include)

# rpihal
include_directories(../../sdk/rpihal/include)
if(_DEBUG)
    add_definitions(-DRPIHAL_CONFIG_LOG_LEVEL=0)
else()
    add_definitions(-DRPIHAL_CONFIG_LOG_LEVEL=0)
endif()
if(NOT PLAT_IS_RASPI)
    add_definitions(-DRPIHAL_EMU)
    set(RPIHAL_CMAKE_CONFIG_EMU true)
endif()
add_subdirectory(../../sdk/rpihal/build/cmake/librpihal.a/ ../../sdk/rpihal/build/cmake/librpihal.a/)


#
# the application
#

set(BINNAME vacuul)

if(_DEBUG)
    add_definitions(-D_DEBUG)
    add_definitions(-DCONFIG_LOG_LEVEL=4)
else()
    add_definitions(-DCONFIG_LOG_LEVEL=0)
endif()

include_directories(../../src/)

set(SOURCES
../../src/main.cpp
../../src/api/api.cpp
../../src/api/data.cpp
../../src/application/appMain.cpp
../../src/application/commissioning.cpp
../../src/application/config.cpp
../../src/application/dfu.cpp
../../src/application/idle.cpp
../../src/application/localSetup.cpp
../../src/application/status.cpp
../../src/application/statusData.cpp
../../src/application/treatment.cpp
../../src/application/treatmentData.cpp
../../src/deviceabstraction/ads1015.cpp
../../src/deviceabstraction/ds1077l.cpp
../../src/deviceabstraction/max32664.cpp
../../src/deviceabstraction/mcp4728.cpp
../../src/deviceabstraction/mcp9808.cpp
../../src/middleware/biometric/max32664.cpp
../../src/middleware/asio-impl.cpp
../../src/middleware/curlThread.cpp
../../src/middleware/display.cpp
../../src/middleware/gpio.cpp
../../src/middleware/hardwareVersion.cpp
../../src/middleware/i2cThread.cpp
../../src/middleware/iniFile.cpp
../../src/middleware/json.cpp
../../src/middleware/otp.cpp
../../src/middleware/theme-light.cpp
../../src/middleware/thread.cpp
../../src/middleware/util.cpp
../../src/middleware/websocket.cpp
)



add_executable(${BINNAME} ${SOURCES})
target_link_libraries(${BINNAME} omw rpihal curl pthread)
target_compile_options(${BINNAME} PRIVATE -Wno-psabi -Wall -Werror=return-type -Werror=switch -Werror=reorder -Werror=format)



if(PLAT_IS_RASPI)
    message(STATUS "we are on the Pi :)")
else()
    message("")
    message("########################################")
    message("########################################")
    message("###                                  ###")
    message("###           not on a Pi            ###")
    message("###                                  ###")
    message("########################################")
    message("########################################")
    message("")
endif()

``````


### build/dep_globals.sh
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/build/dep_globals.sh
`relative_path`: build/dep_globals.sh
`format`: Shell Script
`size`: 831   




### build/readme.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/build/readme.md
`relative_path`: build/readme.md
`format`: Arbitrary Binary Data
`size`: 3640   


``````
# Build and Pack

## General Info
All building and packing is done on a Raspberry Pi host, no cross compiling.

Either the [`build.sh`](./build.sh) script or plain CMake, Make and GCC can be used to build the application.
The helper scripts `pack_deb*.sh` can be used to pack the Debian package.

Consult the release notes and see the release commits (tagged) as well as the one or two before and after to see which releases used what compile switches.

If the build configuration is switched ([release](#release) &#x21C4; [debug](#debug)) a full clean has to be done before building: `./build.sh cleanAll`.
While working run `cleanAll` only once to speed up the compile process.

#### Version Numbers
Before a binary which should be packed is built, the versions have to be set accordingly in [dep_vstr.txt](./dep_vstr.txt) and [project.h](../src/project.h). Both have to be always in sync.
The version strings are using the format of [semver](https://semver.org/spec/v2.0.0.html), thus beeing mostly compatible to the version compare method of Debian. Be cautious when using some special strings for _pre-release_ and _build_.

If the Debian package revision number has to be other than 1, it has to be changed in the `pack_deb*.sh` script accordingly.



## Release Package
1. Set correct [version numbers](#version-numbers)
0. Set the desired compile switches, e.g.:
```c
#define PRJ_ENTER_LOCALSETUP_FROM_IDLE (1)
#define PRJ_EN_PREPARE_STATE           (0)
#define PRJ_USE_HTML_IO_EMU            (0)
#define PRJ_START_AT_HANDS_ON          (0)
#define PRJ_PLAIN_RASPI                (0)

#define PRJ_PKG_DEV_DISPDMY (0)
```
3. Build the binary: `./build.sh cleanAll cmake make`
0. If the compiler reports some warnings they have to be reviewed. It has to be decided for each warning if it can be ignored, or if it may lead to an undesired software configuration.
> The warning _"TODO DFU change from localSetup subtask to task in appMain.cpp, create a returnCommand to switch to DFU state"_
> for example can be ignored (see [DFU Process](../readme.md#dfu-process)).

5. Setup pack dependencies
    - See [_./dep_pack/_](./dep_pack/)
    - Run the [VS Code build task](../.vscode/tasks.json) for [_/var/www/html/display_](../doc/paths.md#L8)
0. Create the Debian package: `./pack_deb.sh`
0. Add it to the [index](../dfu/index.jsonc) and upload it to the server
0. Ideally version numbers are increased after a release (not in the release commit) to keep version continuity (DFU)



## Debug
This configuration can be used during development, when working on the software. It has a log enabled.

```sh
./build.sh cleanAll cmaked
./build.sh make run
```
> If the WiFi config and/or reboot function has to be used use `srun` instead. But when not testing/working on these
> functionalities, unprivileged execution is fine.



## `vacuul-dev-dispdmy_<VERSION>.deb`
This package can be run on a plain RasPi, there is no need for a machine. This can be used to test the display webapp and/or the API.
> This configuration can only be executed properly if installed via the Debian package. If it should be executed
> directly, disable `PRJ_PKG_DEV_DISPDMY`, use [Debug](#debug) and enable `PRJ_PLAIN_RASPI`.

Set the desired compile switches:
```c
#define PRJ_ENTER_LOCALSETUP_FROM_IDLE (1)
#define PRJ_EN_PREPARE_STATE           (0)
#define PRJ_USE_HTML_IO_EMU            (1)
#define PRJ_START_AT_HANDS_ON          (0)
#define PRJ_PLAIN_RASPI                (1)

#define PRJ_PKG_DEV_DISPDMY (1)
```

Run the [VS Code build task](../.vscode/tasks.json).

Build and pack:
```txt
./build.sh [cleanAll] cmaked make && pack_deb_dev-dispdmy.sh
```

``````


### build/build.sh
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/build/build.sh
`relative_path`: build/build.sh
`format`: Shell Script
`size`: 3808   




### build/pack_deb_dev.sh
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/build/pack_deb_dev.sh
`relative_path`: build/pack_deb_dev.sh
`format`: Shell Script
`size`: 2199   




### build/dep_pack/readme.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/build/dep_pack/readme.md
`relative_path`: build/dep_pack/readme.md
`format`: Arbitrary Binary Data
`size`: 69   


``````
# Dependencies of the pack scripts

- [Display](./display/readme.md)

``````


### build/dep_pack/display/readme.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/build/dep_pack/display/readme.md
`relative_path`: build/dep_pack/display/readme.md
`format`: Arbitrary Binary Data
`size`: 261   


``````
Files needed by the _pack_ scripts:
- The `dist` directory of the display webapp release

Alternatively, quick and dirty: extract them from the latest release `.deb` package.

These are not added to git, to save repo memory usage! _Maybe add it as sub repo..._

``````


### build/pack_deb.sh
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/build/pack_deb.sh
`relative_path`: build/pack_deb.sh
`format`: Shell Script
`size`: 2592   




### src/middleware/hardwareVersion.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/hardwareVersion.h
`relative_path`: src/middleware/hardwareVersion.h
`format`: Arbitrary Binary Data
`size`: 1813   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2025 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_HARDWAREVERSION_H
#define IG_MIDDLEWARE_HARDWAREVERSION_H

#include <cstddef>
#include <cstdint>
#include <string>


class HardwareVersion
{
public:
    HardwareVersion()
        : m_maj(0), m_min(0)
    {}

    HardwareVersion(int32_t major, int32_t minor)
        : m_maj(major), m_min(minor)
    {}

    HardwareVersion(const char* str)
        : m_maj(-1), m_min(-1)
    {
        set(str);
    }

    HardwareVersion(const std::string& str)
        : m_maj(-1), m_min(-1)
    {
        set(str);
    }

    virtual ~HardwareVersion() {}

    void set(int32_t major, int32_t minor);
    void set(const std::string& str);
    void set(const char* str) { set(std::string(str)); }

    int32_t major() const { return m_maj; }
    int32_t minor() const { return m_min; }

    std::string toString() const { return std::to_string(m_maj) + '.' + std::to_string(m_min); }

    int compare(const HardwareVersion& b) const;

    bool isValid() const;

protected:
    int32_t m_maj;
    int32_t m_min;
};

static inline bool operator==(const HardwareVersion& a, const HardwareVersion& b) { return (a.compare(b) == 0); }
static inline bool operator!=(const HardwareVersion& a, const HardwareVersion& b) { return !(a == b); }
static inline bool operator<(const HardwareVersion& a, const HardwareVersion& b) { return (a.compare(b) < 0); }
static inline bool operator>(const HardwareVersion& a, const HardwareVersion& b) { return (b < a); }
static inline bool operator<=(const HardwareVersion& a, const HardwareVersion& b) { return !(a > b); }
static inline bool operator>=(const HardwareVersion& a, const HardwareVersion& b) { return !(a < b); }


#endif // IG_MIDDLEWARE_HARDWAREVERSION_H

``````


### src/middleware/curlThread.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/curlThread.cpp
`relative_path`: src/middleware/curlThread.cpp
`format`: Arbitrary Binary Data
`size`: 11699   




### src/middleware/iniFile.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/iniFile.cpp
`relative_path`: src/middleware/iniFile.cpp
`format`: Arbitrary Binary Data
`size`: 11762   




### src/middleware/thread.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/thread.cpp
`relative_path`: src/middleware/thread.cpp
`format`: Arbitrary Binary Data
`size`: 253   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

#include "thread.h"

#include <omw/string.h>


namespace {}


//...

``````


### src/middleware/util.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/util.cpp
`relative_path`: src/middleware/util.cpp
`format`: Arbitrary Binary Data
`size`: 8948   




### src/middleware/websocket.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/websocket.h
`relative_path`: src/middleware/websocket.h
`format`: Arbitrary Binary Data
`size`: 4076   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_WEBSOCKET_H
#define IG_MIDDLEWARE_WEBSOCKET_H

#include <cstddef>
#include <cstdint>
#include <mutex>
#include <string>
#include <vector>

#include <websocketpp/config/asio_no_tls.hpp>
#include <websocketpp/server.hpp>


#define ___WEBSOCKET_BASIC_IMPLEMENTATION (1)


namespace websocket {

#ifdef ___WEBSOCKET_BASIC_IMPLEMENTATION

typedef websocketpp::server<websocketpp::config::asio> server_t;
typedef websocketpp::connection_hdl conn_hdl_t;
typedef server_t::message_ptr msg_ptr_t;
typedef websocketpp::frame::opcode::value opcode;

typedef void (*open_callback_t)(server_t*, conn_hdl_t);
typedef void (*close_callback_t)(server_t*, conn_hdl_t);
typedef void (*message_callback_t)(server_t*, conn_hdl_t, msg_ptr_t);

class Connections
{
public:
    using container_t = std::vector<websocket::conn_hdl_t>;
    using value_t = container_t::value_type;

private:
    using mutex = std::mutex;
    using lock_guard = std::lock_guard<mutex>;

public:
    Connections()
        : m_conn()
    {}

    virtual ~Connections() {}

    // clang-format off
    container_t get() const { lock_guard lg(m_mtx); return m_conn; }
    void add(const value_t& hdl) { lock_guard lg(m_mtx); m_conn.push_back(hdl); }
    // clang-format on

    void remove(const value_t& hdl);
    void clear();

private:
    mutable mutex m_mtx;
    container_t m_conn;

private:
    Connections(const Connections& other) = delete;
    Connections(const Connections&& other) = delete;
    void operator=(const Connections& b) {}
};

void init(uint16_t port);
void run();
void stop();
void register_cb(open_callback_t open, close_callback_t close, message_callback_t message);

Connections::container_t getConnections();

void send(const std::string& data, conn_hdl_t hdl, opcode op = opcode::TEXT);
void send(const void* data, size_t count, conn_hdl_t hdl, opcode op);

void broadcast(const std::string& data, opcode op = opcode::TEXT);
void broadcast(const void* data, size_t count, opcode op);


#else  // ___WEBSOCKET_BASIC_IMPLEMENTATION
class Server
{
public:
    typedef websocketpp::server<websocketpp::config::asio> server_t;
    typedef websocketpp::connection_hdl conn_hdl_t;
    typedef server_t::message_ptr msg_ptr_t;
    typedef websocketpp::frame::opcode::value opcode;

    typedef void (*open_callback_t)(server_t*, conn_hdl_t);
    typedef void (*close_callback_t)(server_t*, conn_hdl_t);
    typedef void (*message_callback_t)(server_t*, conn_hdl_t, msg_ptr_t);

public:
    explicit Server(uint16_t port);
    virtual ~Server() {}

    void init();
    void run();

    void stop();

    void send(const std::string& data, conn_hdl_t hdl, opcode op = opcode::TEXT);
    void send(const void* data, size_t count, conn_hdl_t hdl, opcode op);

    void broadcast(const std::string& data, opcode op = opcode::TEXT);
    void broadcast(const void* data, size_t count, opcode op);

    void broadcastExcept(const std::string& data, conn_hdl_t hdl, opcode op = opcode::TEXT);
    void broadcastExcept(const void* data, size_t count, conn_hdl_t hdl, opcode op);

    void register_open_cb(open_callback_t cb) { m_open_cb = cb; }
    void register_close_cb(close_callback_t cb) { m_close_cb = cb; }
    void register_message_cb(message_callback_t cb) { m_message_cb = cb; }

private:
    server_t m_server;
    uint16_t m_port;
    std::vector<conn_hdl_t> m_connections;

    open_callback_t m_open_cb = nullptr;
    close_callback_t m_close_cb = nullptr;
    message_callback_t m_message_cb = nullptr;

    void m_open_handler(server_t* s, conn_hdl_t hdl);
    void m_close_handler(server_t* s, conn_hdl_t hdl);
    void m_message_handler(server_t* s, conn_hdl_t hdl, msg_ptr_t msg);

private:
    Server() = delete;
    Server(const Server& other) = delete;
    Server(const Server&& other) = delete;
    const Server& operator=(const Server& b) {}
};
#endif // ___WEBSOCKET_BASIC_IMPLEMENTATION

} // namespace websocket


#endif // IG_MIDDLEWARE_WEBSOCKET_H

``````


### src/middleware/asio-impl.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/asio-impl.cpp
`relative_path`: src/middleware/asio-impl.cpp
`format`: Arbitrary Binary Data
`size`: 149   


``````

/*

    see https://think-async.com/Asio/asio-1.30.2/doc/asio/using.html#asio.using.optional_separate_compilation

*/

#include <asio/impl/src.hpp>

``````


### src/middleware/gpio.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/gpio.cpp
`relative_path`: src/middleware/gpio.cpp
`format`: Arbitrary Binary Data
`size`: 6002   




### src/middleware/display.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/display.cpp
`relative_path`: src/middleware/display.cpp
`format`: Arbitrary Binary Data
`size`: 10337   




### src/middleware/otp.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/otp.cpp
`relative_path`: src/middleware/otp.cpp
`format`: Arbitrary Binary Data
`size`: 2401   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2025 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <filesystem>
#include <string>

#include "middleware/iniFile.h"
#include "otp.h"


#define LOG_MODULE_LEVEL LOG_LEVEL_DBG
#define LOG_MODULE_NAME  OTP
#include "middleware/log.h"


namespace fs = std::filesystem;



static const char* const otpFile = "/etc/vacuul-otp.ini";

static const char* const section_hardware = "Hardware";
static const char* const key_hw_version = "Version";

static otp::Data otpData = otp::Data();



int setOtpData(const mw::IniFile& iniFile);



int otp::read()
{
    int r = -1;

    if (fs::exists(otpFile))
    {
        mw::IniFile iniFile;
        iniFile.setFileName(otpFile);
        iniFile.setLineEnding("\n");
        iniFile.setWriteBom(false);

        const int err = iniFile.readFile();
        if (err)
        {
            LOG_ERR("failed to read ini file (%i)", err);
            r = -(__LINE__);
        }
        else { r = setOtpData(iniFile); }
    }
    else // the OTP file does not exist on images from 2024
    {
        // the machine this is running on is a machine that was released in December 2024 or January 2025 and has a hardware version of 2.2
        otpData = otp::Data(HardwareVersion(2, 2));

        LOG_WRN("OTP file does not exist, assuming hw v%s", otp::data->hardwareVersion().toString().c_str());
        r = 0;
    }

    otpData.setValidity(r == 0);

    return r;
}



const otp::Data* const otp::data = &otpData;



int setOtpData(const mw::IniFile& iniFile)
{
    int r = 0;

    // init values as invalid
    HardwareVersion hwv(-1, -1);
    // Type anotherValue;



    // get values from OTP file data

    const char* section = section_hardware;
    const char* key = key_hw_version;
    try
    {
        hwv = iniFile.getValue(section, key);
        if (!hwv.isValid()) { throw(-1); }
    }
    catch (...)
    {
        LOG_ERR("failed to get value \"[%s].%s\"", section, key);
        r = -(__LINE__);
    }

#if 0
    const char* key = key_hw_anotherValue;
    try
    {
        anotherValue = iniFile.getValue(section, key);
    }
    catch (...)
    {
        LOG_ERR("failed to get value \"[%s].%s\"", section, key);
        r = -(__LINE__);
    }
#endif



    // set global OTP data
    otpData = otp::Data(hwv /* , anotherValue */);

    return r;
}

``````


### src/middleware/iniFile.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/iniFile.h
`relative_path`: src/middleware/iniFile.h
`format`: Arbitrary Binary Data
`size`: 3787   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_INIFILE_H
#define IG_INIFILE_H

#include <stdexcept>
#include <string>
#include <vector>

namespace mw {
class IniFile
{
public:
    class entry_not_found : public std::runtime_error
    {
    public:
        entry_not_found() = delete;
        explicit entry_not_found(const std::string& message)
            : std::runtime_error(message)
        {}
        explicit entry_not_found(const char* message)
            : std::runtime_error(message)
        {}
        virtual ~entry_not_found() {}
    };

    class section_not_found : public entry_not_found
    {
    public:
        section_not_found() = delete;
        explicit section_not_found(const std::string& message)
            : entry_not_found(message)
        {}
        explicit section_not_found(const char* message)
            : entry_not_found(message)
        {}
        virtual ~section_not_found() {}
    };

    class key_not_found : public entry_not_found
    {
    public:
        key_not_found() = delete;
        explicit key_not_found(const std::string& message)
            : entry_not_found(message)
        {}
        explicit key_not_found(const char* message)
            : entry_not_found(message)
        {}
        virtual ~key_not_found() {}
    };

    class invalid_file : public std::runtime_error
    {
    public:
        invalid_file() = delete;
        explicit invalid_file(const std::string& message)
            : std::runtime_error(message)
        {}
        explicit invalid_file(const char* message)
            : std::runtime_error(message)
        {}
        virtual ~invalid_file() {}
    };

public:
    IniFile();
    IniFile(const std::string& fileName, const std::string& lineEnding = "\n", bool writeBOM = false);

    const std::string& getFileName() const;
    const std::string& getValue(const std::string& key) const;
    const std::string& getValue(const std::string& section, const std::string& key) const;
    const std::string& getValueD(const std::string& key, const std::string& defaultValue);
    const std::string& getValueD(const std::string& section, const std::string& key, const std::string& defaultValue);

    void setFileName(const std::string& fileName);
    void setLineEnding(const std::string& lineEnding);
    void setWriteBom(bool state) { wrBOM = state; }
    void setValue(const std::string& key, const std::string& value);
    void setValue(const std::string& section, const std::string& key, const std::string& value);

    int readFile();
    int writeFile() const;

    std::string dump() const;

private:
    class Pair
    {
    public:
        Pair() = delete;
        Pair(std::string key, std::string value);

        bool eq(const std::string& key) const;

        std::string k;
        std::string v;
    };

    class Section
    {
    public:
        Section() = delete;
        Section(const std::string& sectionName, const mw::IniFile::Pair& pair);
        Section(const std::string& sectionName, const std::vector<mw::IniFile::Pair>& pair = std::vector<mw::IniFile::Pair>());

        const bool eq(const std::string& sectionName) const;
        std::string& getValue(const std::string& key);
        const std::string& getValue(const std::string& key) const;

        std::string name;
        std::vector<mw::IniFile::Pair> pairs;
    };

private:
    std::string file;
    std::vector<mw::IniFile::Section> sections;
    bool wrBOM;
    char le[3];

    mw::IniFile::Section& getSection(const std::string& sectionName);
    const mw::IniFile::Section& getSection(const std::string& sectionName) const;
    size_t getSectionPos(const std::string& sectionName) const;
};
} // namespace mw

#endif // IG_INIFILE_H

``````


### src/middleware/display.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/display.h
`relative_path`: src/middleware/display.h
`format`: Arbitrary Binary Data
`size`: 4103   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_DISPLAY_H
#define IG_MIDDLEWARE_DISPLAY_H

#include <cstddef>
#include <cstdint>

#include "middleware/thread.h"


namespace display {

class LocalSetupAction
{
public:
    enum class Type
    {
        none = 0,

        // system
        updateDispBright, // update display brightness, only current value (for user to see the change)
        reboot,
        swUpdate,
        saveSystem,

        // WiFi
        saveWiFi,
    };

private:
    static constexpr int m_dispBrightDontCare = 100;

public:
    LocalSetupAction()
        : m_updateIdx(0), m_type(Type::none), m_displayBrightness(m_dispBrightDontCare), m_wifiCountry(), m_wifiSsid(), m_wifiPsk()
    {}

    LocalSetupAction(int updateIdx, Type type)
        : m_updateIdx(updateIdx), m_type(type), m_displayBrightness(m_dispBrightDontCare), m_wifiCountry(), m_wifiSsid(), m_wifiPsk()
    {}

    LocalSetupAction(int updateIdx, Type type, int displayBrightness)
        : m_updateIdx(updateIdx), m_type(type), m_displayBrightness(displayBrightness), m_wifiCountry(), m_wifiSsid(), m_wifiPsk()
    {}

    LocalSetupAction(int updateIdx, const std::string& wifiCountry, const std::string& wifiSsid, const std::string& wifiPsk)
        : m_updateIdx(updateIdx),
          m_type(Type::saveWiFi),
          m_displayBrightness(m_dispBrightDontCare),
          m_wifiCountry(wifiCountry),
          m_wifiSsid(wifiSsid),
          m_wifiPsk(wifiPsk)
    {}

    virtual ~LocalSetupAction() {}

    const int& updateIdx() const { return m_updateIdx; }
    const Type& type() const { return m_type; }

    const int& displayBrightness() const { return m_displayBrightness; }

    const std::string& wifiCountry() const { return m_wifiCountry; }
    const std::string& wifiSsid() const { return m_wifiSsid; }
    const std::string& wifiPsk() const { return m_wifiPsk; }

private:
    int m_updateIdx;
    Type m_type;

    int m_displayBrightness;

    std::string m_wifiCountry;
    std::string m_wifiSsid;
    std::string m_wifiPsk;
};

class LocalSetupAnswer
{
public:
    enum class Area
    {
        undefined = 0,
        system,
        wifi,
    };

public:
    LocalSetupAnswer()
        : m_area(Area::undefined), m_error(-1), m_msg()
    {}

    LocalSetupAnswer(const Area& area, int error, const std::string& msg)
        : m_area(area), m_error(error), m_msg(msg)
    {}

    virtual ~LocalSetupAnswer() {}

    const Area& area() const { return m_area; }
    std::string areaStr() const;
    int error() const { return m_error; }
    const std::string& msg() const { return m_msg; }

private:
    Area m_area;
    int m_error;
    std::string m_msg;
};

class ThreadSharedData : public ::thread::SharedData
{
public:
    ThreadSharedData() {}
    virtual ~ThreadSharedData() {}

    // clang-format off
    void shutdown(bool state = true) { lock_guard lg(m_mtx); m_shutdown = state; }
    bool testShutdown() const { lock_guard lg(m_mtx); return m_shutdown; }
    // clang-format on

    // clang-format off
    void set(const LocalSetupAction& data) { lock_guard lg(m_mtx); m_localSetupAction = data; }
    void answer(const LocalSetupAnswer& ans) { lock_guard lg(m_mtx); m_localSetupAnswer = ans; ++m_answerUpdateIdx; }

    LocalSetupAction getLocalSetupAction() const { lock_guard lg(m_mtx); return m_localSetupAction; }
    LocalSetupAnswer getLocalSetupAnswer() const { lock_guard lg(m_mtx); return m_localSetupAnswer; }

    int getAnswerUpdateIdx() const { lock_guard lg(m_mtx); return m_answerUpdateIdx; }

    void setWsConnState(bool state) { lock_guard lg(m_mtx); m_wsConnected = state; }
    bool getWsConnState() const { lock_guard lg(m_mtx); return m_wsConnected; }
    // clang-format on

private:
    bool m_shutdown = false;

    LocalSetupAction m_localSetupAction;
    LocalSetupAnswer m_localSetupAnswer;
    int m_answerUpdateIdx = 0;
    bool m_wsConnected = false;
};

extern ThreadSharedData sd;

void thread();

} // namespace display


#endif // IG_MIDDLEWARE_DISPLAY_H

``````


### src/middleware/hardwareVersion.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/hardwareVersion.cpp
`relative_path`: src/middleware/hardwareVersion.cpp
`format`: Arbitrary Binary Data
`size`: 1332   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2025 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <string>

#include "hardwareVersion.h"

#include <omw/string.h>


void HardwareVersion::set(int32_t major, int32_t minor)
{
    m_maj = major;
    m_min = minor;
}

void HardwareVersion::set(const std::string& str)
{
    try
    {
        const auto tokens = omw::split(str, '.');
        if ((tokens.size() == 2) && omw::isUInteger(tokens[0]) && omw::isUInteger(tokens[1])) { set(std::stoi(tokens[0]), std::stoi(tokens[1])); }
        else { set(-1, -1); }
    }
    catch (...)
    {
        set(-1, -1);
    }
}

/**
 * @brief Compares to another version.
 *
 * | Result         | Return Value |
 * |:--------------:|:---:|
 * | `*this` < `b`  | <0  |
 * | `*this` == `b` |  0  |
 * | `*this` > `b`  | >0  |
 *
 * @param b The other version
 */
int HardwareVersion::compare(const HardwareVersion& b) const
{
    int r = 0;

    if (this->major() < b.major()) { r = -1; }
    else if (this->major() == b.major())
    {
        if (this->minor() < b.minor()) { r = -1; }
        else if (this->minor() == b.minor()) { r = 0; }
        else { r = 1; }
    }
    else { r = 1; }

    return r;
}

bool HardwareVersion::isValid() const { return (m_maj >= 0) && (m_min >= 0); }

``````


### src/middleware/thread.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/thread.h
`relative_path`: src/middleware/thread.h
`format`: Arbitrary Binary Data
`size`: 1013   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_THREAD_H
#define IG_MIDDLEWARE_THREAD_H

#include <cstddef>
#include <cstdint>
#include <mutex>


namespace thread {

class SharedData
{
public:
    using lock_guard = std::lock_guard<std::mutex>;

public:
    SharedData()
        : m_booted(false), m_terminate(false)
    {}

    virtual ~SharedData() {}

    // clang-format off
    void setBooted(bool state = true) { lock_guard lg(m_mtxThCtrl); m_booted = state; }
    void terminate(bool state = true) { lock_guard lg(m_mtxThCtrl); m_terminate = state; }
    bool isBooted() const { lock_guard lg(m_mtxThCtrl); return m_booted; }
    bool testTerminate() const { lock_guard lg(m_mtxThCtrl); return m_terminate; }
    // clang-format on

protected:
    mutable std::mutex m_mtx;
    bool m_booted;
    bool m_terminate;

private:
    mutable std::mutex m_mtxThCtrl;
};

} // namespace thread


#endif // IG_MIDDLEWARE_THREAD_H

``````


### src/middleware/otp.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/otp.h
`relative_path`: src/middleware/otp.h
`format`: Arbitrary Binary Data
`size`: 1002   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2025 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_OTP_H
#define IG_MIDDLEWARE_OTP_H

#include <cstddef>
#include <cstdint>

#include "middleware/hardwareVersion.h"


namespace otp {

class Data
{
public:
    Data()
        : m_validity(false), m_hwv(0, 0)
    {}

    explicit Data(const HardwareVersion& hwv)
        : m_validity(false), m_hwv(hwv)
    {}

    void setValidity(bool validity) { m_validity = validity; }

    const HardwareVersion& hardwareVersion() const { return m_hwv; }

    // returns `true` only if all data is valid
    bool isValid() const { return m_validity; }

private:
    bool m_validity;
    HardwareVersion m_hwv;
};

/**
 * @brief Reads the OPT file.
 *
 * If the file does not exist, it's assumed that the hardware version is `v2.2` and no error is returned.
 *
 * @return 0 on success
 */
int read();

extern const Data* const data;

} // namespace otp


#endif // IG_MIDDLEWARE_OTP_H

``````


### src/middleware/biometric/data.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/biometric/data.h
`relative_path`: src/middleware/biometric/data.h
`format`: Arbitrary Binary Data
`size`: 1321   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_BIOMETRIC_DATA_H
#define IG_MIDDLEWARE_BIOMETRIC_DATA_H

#include <cstddef>
#include <cstdint>


namespace biometric {



enum class MAX32664ChipVersion
{
    unknown = 0,
    A,
    B,
    C,
    D,
};
const char* const toString(MAX32664ChipVersion v);

enum class AFEType
{
    unknown = 0,
    MAX3010x,
};
const char* const toString(AFEType afe_type);



class Data
{
public:
    static constexpr int algStatUnknown = -999999; // unknown/undefined value for `algState` and `algStatus`

public:
    Data()
        : m_hr(0), m_oxySat(0), m_algState(algStatUnknown), m_algStatus(algStatUnknown)
    {}

    Data(float hr, float oxySat, int algState, int algStatus)
        : m_hr(hr), m_oxySat(oxySat), m_algState(algState), m_algStatus(algStatus)
    {}

    virtual ~Data() {}

    // [bpm]
    float heartRate() const { return m_hr; }

    // oxygen saturation [%]
    float oxygenSat() const { return m_oxySat; }

    int algorithmState() const { return m_algState; }
    int algorithmStatus() const { return m_algStatus; }

private:
    float m_hr;
    float m_oxySat;
    int m_algState;
    int m_algStatus;
};

} // namespace biometric


#endif // IG_MIDDLEWARE_BIOMETRIC_DATA_H

``````


### src/middleware/biometric/max32664.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/biometric/max32664.h
`relative_path`: src/middleware/biometric/max32664.h
`format`: Arbitrary Binary Data
`size`: 4061   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_BIOMETRIC_MAX32664_H
#define IG_MIDDLEWARE_BIOMETRIC_MAX32664_H

#include <cstddef>
#include <cstdint>
#include <vector>

#include "deviceabstraction/max32664.h"
#include "middleware/biometric/data.h"

#include <omw/version.h>


namespace biometric {

#if defined(_DEBUG) && 1
#define BIOMETRIC_EMULATOR_AVAILABLE (1)
biometric::Data emulator();
#endif

class MAX32664
{
public:
    MAX32664() = delete;

    MAX32664(int gpioPin_mfio, int gpioPin_nRst, const char* dev, uint8_t addr = deviceabstr::i2c::MAX32664::default_i2c_addr);

    virtual ~MAX32664() {}



    const MAX32664ChipVersion& chipVersion() const { return m_chipVersion; }

    const Data& data() const { return m_data; }
    void clearValues() { m_data = Data(); }



    void task();

    void forceReboot();

    bool ctorOk() const;
    bool booted() const { return m_booted; }

private:
    Data m_data;



private:
    MAX32664ChipVersion m_chipVersion = MAX32664ChipVersion::unknown;
    uint8_t m_hub_type = (-1);
    omw::Version m_hub_version = "0.0.0";
    uint8_t m_hub_outputFormat = 0;
    uint8_t m_hub_algMode = 0;         // WHRM disabled, normal, extended
    size_t m_hub_sampleSize = 1;       // number of bytes of one single output sample block
    size_t m_hub_sampleAlgDataOff = 0; // algorithm data offset in the sample
    size_t m_hub_nOfAvailSamples = 0;
    uint8_t m_afe_partID = 0;     // AFE register FF
    uint8_t m_afe_revisionID = 0; // AFE register FE
    bool m_accelEnabled = false;

    int m_state;   // init in ctor
    bool m_booted; // init in ctor
    int64_t m_tpAction = 0;

    int m_subtask_state = 0;
    int64_t m_subtask_tpAct = 0;
    int64_t m_subtask_tp2nd = 0;
    int m_subtask_rstApp(const int64_t& tpNow); // hw reset to application
    int m_subtask_boot(const int64_t& tpNow);

    size_t m_boot_deviceModeAttempt;

    std::vector<uint8_t> m_dfu_fileData;
    size_t m_dfu_nPages;
    size_t m_dfu_pageIdx;
    size_t m_dfu_pageSize;



private:
    deviceabstr::i2c::MAX32664 m_dev;
    std::array<uint8_t, MAX32664_DATA_BUFFER_SIZE> m_buffer;

    // trx may be subsubtask so separate state and action timepoints are needed
    int m_subtask_trx_state = 0;
    int64_t m_subtask_trx_tpAct = 0;
    bool m_subtask_trx_prohibitLog = false; // ommits logging for the next and only the next transaction

    /**
     * @brief Performs a read transaction.
     *
     * Read data is stored in `m_buffer`.
     *
     * @param family Command family byte
     * @param index Command index byte
     * @param count Number of data bytes to be read
     * @param txData Optional write data
     * @param txCount Size of write data
     * @return One of the three default subtask return values `subtask_...`
     */
    int m_subtask_trxRead(uint8_t family, uint8_t index, size_t count, const uint8_t* txData = nullptr, size_t txCount = 0);
    int m_subtask_trxRead(uint8_t family, uint8_t index, uint8_t writeByte0, size_t count) { return m_subtask_trxRead(family, index, count, &writeByte0, 1); }

    /**
     * @brief Performs a write transaction.
     *
     * Data from `m_buffer` is written.
     *
     * @param family Command family byte
     * @param index Command index byte
     * @param count Number of data bytes to be written
     * @return One of the three default subtask return values `subtask_...`
     */
    int m_subtask_trxWrite(uint8_t family, uint8_t index, size_t count);
    int m_subtask_trxWriteByte(uint8_t family, uint8_t index, uint8_t writeByte0)
    {
        m_buffer[0] = writeByte0;
        return m_subtask_trxWrite(family, index, 1);
    }

#ifdef _DEBUG
private:
    int m_dbg_oldState = -1;
    std::string m_dbg_stateStr;
#endif // _DEBUG

private:
    MAX32664(const MAX32664& other) = delete;
    MAX32664(const MAX32664&& other) = delete;
    MAX32664& operator=(const MAX32664& other);
};

} // namespace biometric


#endif // IG_MIDDLEWARE_BIOMETRIC_MAX32664_H

``````


### src/middleware/biometric/max32664.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/biometric/max32664.cpp
`relative_path`: src/middleware/biometric/max32664.cpp
`format`: Arbitrary Binary Data
`size`: 52939   




### src/middleware/log.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/log.h
`relative_path`: src/middleware/log.h
`format`: Arbitrary Binary Data
`size`: 3525   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_LOG_H
#define IG_MIDDLEWARE_LOG_H

#include <cstdio>



#define LOG_LEVEL_OFF (0)
#define LOG_LEVEL_ERR (1)
#define LOG_LEVEL_WRN (2)
#define LOG_LEVEL_INF (3)
#define LOG_LEVEL_DBG (4)

#ifndef CONFIG_LOG_LEVEL
#warning "CONFIG_LOG_LEVEL is not defined, defaulting to 2 (warning)"
#define CONFIG_LOG_LEVEL LOG_LEVEL_WRN
#endif

#ifndef LOG_MODULE_LEVEL
#error "define LOG_MODULE_LEVEL before including log.h"
#endif
#ifndef LOG_MODULE_NAME
#error "define LOG_MODULE_NAME before including log.h"
#endif



// SGR foreground colors
#define LOG_SGR_BLACK    "\033[30m"
#define LOG_SGR_RED      "\033[31m"
#define LOG_SGR_GREEN    "\033[32m"
#define LOG_SGR_YELLOW   "\033[33m"
#define LOG_SGR_BLUE     "\033[34m"
#define LOG_SGR_MAGENTA  "\033[35m"
#define LOG_SGR_CYAN     "\033[36m"
#define LOG_SGR_WHITE    "\033[37m"
#define LOG_SGR_DEFAULT  "\033[39m"
#define LOG_SGR_BBLACK   "\033[90m"
#define LOG_SGR_BRED     "\033[91m"
#define LOG_SGR_BGREEN   "\033[92m"
#define LOG_SGR_BYELLOW  "\033[93m"
#define LOG_SGR_BBLUE    "\033[94m"
#define LOG_SGR_BMAGENTA "\033[95m"
#define LOG_SGR_BCYAN    "\033[96m"
#define LOG_SGR_BWHITE   "\033[97m"



// optional args
#define ___LOG_OPT_VA_ARGS(...) , ##__VA_ARGS__

// stringify
#define ___LOG_STR_HELPER(x) #x
#define ___LOG_STR(x)        ___LOG_STR_HELPER(x)

#define ___LOG_CSI_EL "\033[2K" // ANSI ESC CSI erase line



// config can limit log level
#if (CONFIG_LOG_LEVEL < LOG_MODULE_LEVEL)
#undef LOG_MODULE_LEVEL
#define LOG_MODULE_LEVEL CONFIG_LOG_LEVEL
#endif



#include "middleware/util.h"

// clang-format off
#define LOG_ERR(msg, ...) printf((___LOG_CSI_EL "[" + util::t_to_iso8601_local(std::time(nullptr)) + "] " "\033[91m" ___LOG_STR(LOG_MODULE_NAME) " <ERR> " msg "\033[39m" "\n").c_str() ___LOG_OPT_VA_ARGS(__VA_ARGS__))
#define LOG_WRN(msg, ...) printf((___LOG_CSI_EL "[" + util::t_to_iso8601_local(std::time(nullptr)) + "] " "\033[93m" ___LOG_STR(LOG_MODULE_NAME) " <WRN> " msg "\033[39m" "\n").c_str() ___LOG_OPT_VA_ARGS(__VA_ARGS__))
#define LOG_INF(msg, ...) printf((___LOG_CSI_EL "[" + util::t_to_iso8601_local(std::time(nullptr)) + "] " "\033[39m" ___LOG_STR(LOG_MODULE_NAME) " <INF> " msg "\033[39m" "\n").c_str() ___LOG_OPT_VA_ARGS(__VA_ARGS__))
//#define LOG_DBG(msg, ...) printf((___LOG_CSI_EL "[" + util::t_to_iso8601_local(std::time(nullptr)) + "] " "\033[39m" ___LOG_STR(LOG_MODULE_NAME) " <DBG> " msg "\033[39m" "\n").c_str() ___LOG_OPT_VA_ARGS(__VA_ARGS__))
#define LOG_DBG(msg, ...) printf((___LOG_CSI_EL "[" + util::t_to_iso8601_local(std::time(nullptr)) + "] " "\033[39m" ___LOG_STR(LOG_MODULE_NAME) " <DBG> \033[90m" + std::string(__func__) +  "():" + std::to_string(__LINE__) + "\033[39m " msg "\033[39m" "\n").c_str() ___LOG_OPT_VA_ARGS(__VA_ARGS__))
// clang-format on



#if (LOG_MODULE_LEVEL < LOG_LEVEL_DBG)
#undef LOG_DBG
#define LOG_DBG(...) (void)0
#endif
#if (LOG_MODULE_LEVEL < LOG_LEVEL_INF)
#undef LOG_INF
#define LOG_INF(...) (void)0
#endif
#if (LOG_MODULE_LEVEL < LOG_LEVEL_WRN)
#undef LOG_WRN
#define LOG_WRN(...) (void)0
#endif
#if (LOG_MODULE_LEVEL < LOG_LEVEL_ERR)
#undef LOG_ERR
#define LOG_ERR(...) (void)0
#endif



#include <unistd.h>
#define LOG_invalidState(_state, _t_s)       \
    {                                        \
        LOG_ERR("invalid state %i", _state); \
        usleep(_t_s * 1000 * 1000);          \
    }
// end LOG_invalidState



#endif // IG_MIDDLEWARE_LOG_H

``````


### src/middleware/json.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/json.h
`relative_path`: src/middleware/json.h
`format`: Arbitrary Binary Data
`size`: 2669   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_JSON_H
#define IG_MIDDLEWARE_JSON_H



// #pragma GCC diagnostic push
// #pragma GCC diagnostic ignored "-Wpsabi" // does not help :(
//                                             needed to add it in CMake:
//                                             target_compile_options(${BINNAME} PRIVATE -Wno-psabi -W...)

// https://stackoverflow.com/a/48149400
#include <json/json.hpp>

// #pragma GCC diagnostic pop



using json = nlohmann::json;



class JsonTypeCheckItem
{
public:
    JsonTypeCheckItem() = delete;

    JsonTypeCheckItem(const std::string& key, const json::value_t& type)
        : m_key(key), m_nDiffTypes(1), m_type0(type), m_type1(type), m_type2(type)
    {}

    JsonTypeCheckItem(const std::string& key, const json::value_t& type0, const json::value_t& type1)
        : m_key(key), m_nDiffTypes(2), m_type0(type0), m_type1(type1), m_type2(type1)
    {}

    JsonTypeCheckItem(const std::string& key, const json::value_t& type0, const json::value_t& type1, const json::value_t& type2)
        : m_key(key), m_nDiffTypes(3), m_type0(type0), m_type1(type1), m_type2(type2)
    {}

    const std::string& key() const { return m_key; }
    size_t nDiffTypes() const { return m_nDiffTypes; }
    const json::value_t& type0() const { return m_type0; }
    const json::value_t& type1() const { return m_type1; }
    const json::value_t& type2() const { return m_type2; }

    bool checkType(const json::value_t& type) const { return ((type == m_type0) || (type == m_type1) || (type == m_type2)); }

private:
    std::string m_key;
    size_t m_nDiffTypes;
    json::value_t m_type0;
    json::value_t m_type1;
    json::value_t m_type2;
};



std::string jsonTypeCheck(const json& data, const JsonTypeCheckItem& check);

/**
 * @brief
 *
 * @param data The parent JSON object
 * @param key Key of the value to be checked
 * @param type Expected type
 * @return Exception message, empty if OK
 */
static inline std::string jsonTypeCheck(const json& data, const std::string& key, json::value_t type)
{
    return jsonTypeCheck(data, JsonTypeCheckItem(key, type));
}

static inline std::string jsonTypeCheck(const json& data, const std::string& key, json::value_t type0, json::value_t type1)
{
    return jsonTypeCheck(data, JsonTypeCheckItem(key, type0, type1));
}

static inline std::string jsonTypeCheck(const json& data, const std::string& key, json::value_t type0, json::value_t type1, json::value_t type2)
{
    return jsonTypeCheck(data, JsonTypeCheckItem(key, type0, type1, type2));
}


#endif // IG_MIDDLEWARE_JSON_H

``````


### src/middleware/gpio.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/gpio.h
`relative_path`: src/middleware/gpio.h
`format`: Arbitrary Binary Data
`size`: 4237   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_GPIO_H
#define IG_MIDDLEWARE_GPIO_H

#include "project.h"

#include <rpihal/gpio.h>


#define GPIO_LED_RUN    (25)
#define GPIO_LED_STAT_1 (27)
#define GPIO_LED_STAT_2 (26)

#define GPIO_PELTPWR_L (14)
#define GPIO_PELTPWR_R (15)

#define GPIO_FAN_GRP_A (19)
#define GPIO_FAN_GRP_B (17)
#define GPIO_FAN_GRP_C (18)
#define GPIO_FAN_PELT  (16)

#define GPIO_PUMP_A (20)
#define GPIO_PUMP_B (21)

#define GPIO_HANDS_ON (24)

#define GPIO_DISP_BACKL_EN (23) // display backlight enable

// pins used for MAX32664 are defined in i2cThread.cpp



#define GPIO_EN_RESIN0 (0)

#if GPIO_EN_RESIN0
// reserved 6, 7, 22
#define GPIO_RESIN0 (6)
#endif // GPIO_EN_RESIN0



/**
 * @brief Initialises the GPIOs needed by this project.
 *
 * @return 0 on success
 */
int GPIO_init();

int GPIO_failsafe();

/**
 * @brief Deinitialises only the GPIOs initialised by this project
 *
 * @return 0 on success
 */
int GPIO_reset();



namespace gpio {

class EdgeDetect
{
public:
    EdgeDetect()
        : m_state(), m_old(), m_pos(), m_neg()
    {}

    EdgeDetect(bool state, bool oldState)
        : m_state(state), m_old(oldState), m_pos(), m_neg()
    {}

    virtual ~EdgeDetect() {}

    bool state() const { return m_state; }
    bool pos() const { return m_pos; }
    bool neg() const { return m_neg; }

    void handler(bool state)
    {
        m_state = state;
        m_pos = (!m_old && m_state);
        m_neg = (m_old && !m_state);
        m_old = m_state;
    }

    operator bool() const { return m_state; }

private:
    bool m_state, m_old, m_pos, m_neg;
};

class Button : public EdgeDetect
{
public:
    Button() = delete;

    explicit Button(int pin)
        : EdgeDetect(), m_pin(pin)
    {
        handler();
        handler();
    }

    virtual ~Button() {}

    virtual void handler() { EdgeDetect::handler(RPIHAL_GPIO_readPin(m_pin) > 0); }

protected:
    int m_pin;
};

class ButtonInverted : public Button
{
public:
    ButtonInverted() = delete;

    explicit ButtonInverted(int pin)
        : Button(pin)
    {
        handler();
        handler();
    }

    virtual ~ButtonInverted() {}

    virtual void handler() { EdgeDetect::handler(RPIHAL_GPIO_readPin(m_pin) == 0); }
};

class Output
{
public:
    Output() = delete;

    explicit Output(int pin)
        : m_pin(pin)
    {}

    // Output(int pin, bool state)          don't do that, will call RPIHAL before RPIHAL init
    //     : m_pin(pin)
    // {
    //     this->write(state);
    // }

    virtual ~Output() {}

    virtual bool read() const = 0;
    virtual void write(bool state) = 0;
    virtual void set() { this->write(1); }
    virtual void clr() { this->write(0); }
    virtual void toggle() { RPIHAL_GPIO_togglePin(m_pin); }

protected:
    int m_pin;

private:
    Output(const Output& other) = delete;
    Output(const Output&& other) = delete;
    Output& operator=(const Output& other);
};

class OutputActivHigh : public Output
{
public:
    OutputActivHigh() = delete;

    explicit OutputActivHigh(int pin)
        : Output(pin)
    {}

    virtual ~OutputActivHigh() {}

    virtual bool read() const { return (RPIHAL_GPIO_readPin(m_pin) > 0); }
    virtual void write(bool state) { RPIHAL_GPIO_writePin(m_pin, state); }
};

class OutputActivLow : public Output
{
public:
    OutputActivLow() = delete;

    explicit OutputActivLow(int pin)
        : Output(pin)
    {}

    virtual ~OutputActivLow() {}

    virtual bool read() const { return (RPIHAL_GPIO_readPin(m_pin) == 0); }
    virtual void write(bool state) { RPIHAL_GPIO_writePin(m_pin, (state ? 0 : 1)); }
};

#if GPIO_EN_RESIN0
extern const EdgeDetect* const resIn0;
#endif // GPIO_EN_RESIN0

namespace led {
    extern Output* const run;
    extern Output* const stat1;
    extern Output* const stat2;
}

extern Output* const peltPowerL;
extern Output* const peltPowerR;
extern Output* const fanA;
extern Output* const fanB;
extern Output* const fanC;
extern Output* const fanPelt;
extern Output* const pumpA;
extern Output* const pumpB;
extern Output* const dispBacklightEn;

void task();

} // namespace gpio


#endif // IG_MIDDLEWARE_GPIO_H

``````


### src/middleware/util.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/util.h
`relative_path`: src/middleware/util.h
`format`: Arbitrary Binary Data
`size`: 5545   




### src/middleware/pid.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/pid.h
`relative_path`: src/middleware/pid.h
`format`: Arbitrary Binary Data
`size`: 2701   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_PID_H
#define IG_MIDDLEWARE_PID_H

#include <cstddef>
#include <cstdint>


namespace pid {

class base_pid
{
public:
    base_pid() = delete;

    base_pid(float kp, float ki, float kd, float lastError = 0.0f, float integral = 0.0f)
        : m_kp(kp), m_ki(ki), m_kd(kd), m_lastError(lastError), m_integral(integral)
    {}

    virtual ~base_pid() {}

    /**
     * @param sp Set Point
     * @param pv Process Value
     * @return Controller Output (CO)
     */
    virtual float proc(float sp, float pv)
    {
        float error = m_calcError(sp, pv);

        const float p = error * m_kp;

        m_integral += error * m_dt;
        const float i = m_integral * m_ki;

        // clamp integral (application speciffic)
        constexpr float intMax = 150.0f;                  // TODO evaluate
        if (m_integral > intMax) { m_integral = intMax; } // TODO report? (could be used to detect sensor or actor fault)
        if (m_integral < -intMax) { m_integral = -intMax; }

        const float derivative = (error - m_lastError) / m_dt;
        const float d = derivative * m_kd;

        const float co = p + i + d;

        // printf("sp: %2.1f, pv: %2.3f, error: %2.3f, p: %2.3f, i: %3.3f, d: %3.3f, co: %3.3f\n", sp, pv, error, p, i, d, co);

        m_lastError = error;

        return co;
    }

protected:
    float m_kp;
    float m_ki;
    float m_kd;
    static constexpr float m_dt = 1.0f; // aka not used
    float m_lastError;
    float m_integral;

    /**
     * @brief Calculates the error in normal or reversed mode.
     *
     * - normal mode: `error = sp - pv`
     * - reversed mode: `error = pv - sp`
     *
     * @param sp Set Point
     * @param pv Process Value
     */
    virtual float m_calcError(float sp, float pv) const = 0;
};

/**
 * @brief Normal mode PID regulator.
 */
class PID : public base_pid
{
public:
    PID() = delete;

    PID(float kp, float ki, float kd, float lastError = 0.0f, float integral = 0.0f)
        : base_pid(kp, ki, kd, lastError, integral)
    {}

    virtual ~PID() {}

protected:
    virtual float m_calcError(float sp, float pv) const { return (sp - pv); }
};

/**
 * @brief Reversed mode PID regulator.
 */
class PIDrev : public base_pid
{
public:
    PIDrev() = delete;

    PIDrev(float kp, float ki, float kd, float lastError = 0.0f, float integral = 0.0f)
        : base_pid(kp, ki, kd, lastError, integral)
    {}

    virtual ~PIDrev() {}

protected:
    virtual float m_calcError(float sp, float pv) const { return (pv - sp); }
};

} // namespace pid


#endif // IG_MIDDLEWARE_PID_H

``````


### src/middleware/websocket.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/websocket.cpp
`relative_path`: src/middleware/websocket.cpp
`format`: Arbitrary Binary Data
`size`: 6338   




### src/middleware/json.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/json.cpp
`relative_path`: src/middleware/json.cpp
`format`: Arbitrary Binary Data
`size`: 1204   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <string>

#include "json.h"


namespace {}



std::string jsonTypeCheck(const json& data, const JsonTypeCheckItem& check)
{
    std::string exWhat;

    try
    {
        const auto value = data.at(check.key());

        if (false == check.checkType(value.type()))
        {
            const auto n = check.nDiffTypes();
            const auto j0 = json(check.type0());
            const auto j1 = json(check.type1());
            const auto j2 = json(check.type2());

            std::string expectedStr = "expected " + std::string(j0.type_name());
            if (n > 1) { expectedStr += " or " + std::string(j1.type_name()); }
            if (n > 2) { expectedStr += " or " + std::string(j2.type_name()); }

            throw std::runtime_error("\"" + check.key() + "\" is " + std::string(value.type_name()) + " (" + expectedStr + ")");
        }
    }
    catch (const std::exception& ex)
    {
        exWhat = ex.what();
    }
    catch (...)
    {
        exWhat = std::string(__func__) + " unknown exception";
    }

    return exWhat;
}

``````


### src/middleware/i2cThread.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/i2cThread.cpp
`relative_path`: src/middleware/i2cThread.cpp
`format`: Arbitrary Binary Data
`size`: 34708   




### src/middleware/theme-light.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/theme-light.h
`relative_path`: src/middleware/theme-light.h
`format`: Arbitrary Binary Data
`size`: 1149   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_THEMELIGHT_H
#define IG_MIDDLEWARE_THEMELIGHT_H

#include <cstddef>
#include <cstdint>

#include "middleware/thread.h"

#include <omw/color.h>


namespace thltThread {

enum class Mode
{
    off,
    commissioning,
    idle,
    treatment,
    error,

    localSetup = idle,
    prepare = idle,
};

class ThreadSharedData : public thread::SharedData
{
public:
    ThreadSharedData()
        : m_mode(Mode::off), m_colour(0)
    {}

    virtual ~ThreadSharedData() {}

    // clang-format off
    void set(const Mode& mode, const omw::Color& colour) { lock_guard lg(m_mtx); m_mode = mode; m_colour = colour; }
    void setMode(const Mode& mode) { lock_guard lg(m_mtx); m_mode = mode; }

    omw::Color getColour() const { lock_guard lg(m_mtx); return m_colour; }
    Mode getMode() const { lock_guard lg(m_mtx); return m_mode; }
    // clang-format on

private:
    Mode m_mode;
    omw::Color m_colour;
};

extern ThreadSharedData sd;

void thread();

} // namespace thltThread


#endif // IG_MIDDLEWARE_THEMELIGHT_H

``````


### src/middleware/curlThread.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/curlThread.h
`relative_path`: src/middleware/curlThread.h
`format`: Arbitrary Binary Data
`size`: 6148   




### src/middleware/i2cThread.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/i2cThread.h
`relative_path`: src/middleware/i2cThread.h
`format`: Arbitrary Binary Data
`size`: 3905   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_MIDDLEWARE_I2CTHREAD_H
#define IG_MIDDLEWARE_I2CTHREAD_H

#include <cstddef>
#include <cstdint>

#include "middleware/biometric/data.h"
#include "middleware/thread.h"
#include "project.h"


namespace i2cThread {


template <typename T> class SetPoint
{
public:
    using value_type = T;

public:
    SetPoint() = delete;

    explicit SetPoint(const value_type& value, bool idle = false)
        : m_idle(idle), m_value(value)
    {}

    explicit SetPoint(bool idle)
        : m_idle(idle), m_value(0)
    {}

    virtual ~SetPoint() {}

    void setIdle() { m_idle = true; }
    void setValue(const value_type& value)
    {
        m_value = value;
        m_idle = false;
    }

    const value_type& value() const { return m_value; }
    bool isIdle() const { return m_idle; }

private:
    bool m_idle;
    value_type m_value;
};

class PeltierTemperatures
{
public:
    PeltierTemperatures() = delete;

    PeltierTemperatures(float left, float right)
        : m_left(left), m_right(right)
    {}

    virtual ~PeltierTemperatures() {}

    float left() const { return m_left; }
    float right() const { return m_right; }

private:
    float m_left;
    float m_right;
};

class ThreadSharedData : public thread::SharedData
{
public:
    ThreadSharedData()
        : m_tempAir(0), m_tempPelt(0, 0), m_biom(), m_tempPeltSp(20.0f), m_treatmentStarted(false), m_treatmentStopped(false), m_dispBrightness(0)
    {
        m_tempPeltSp.setIdle();
    }

    virtual ~ThreadSharedData() {}

    // clang-format off

    void setTempAir(float value) { lock_guard lg(m_mtx); m_tempAir = value; }
    float getTempAir() const { lock_guard lg(m_mtx); return m_tempAir; }

    void setBiometric(const biometric::Data& biom) { lock_guard lg(m_mtx); m_biom = biom; }
    biometric::Data getBiometric() const { lock_guard lg(m_mtx); return m_biom; }

    void setTempPelt(float left, float right) { lock_guard lg(m_mtx); m_tempPelt = PeltierTemperatures(left, right); }
    void setTempPelt(const PeltierTemperatures& data) { lock_guard lg(m_mtx); m_tempPelt = data; }
    const PeltierTemperatures& getTempPelt() const { lock_guard lg(m_mtx); return m_tempPelt; }

    void setTempPeltSp(float value) { lock_guard lg(m_mtx); m_tempPeltSp.setValue(value); }
    SetPoint<float> getTempPeltSp() const { lock_guard lg(m_mtx); return m_tempPeltSp; }

    void setTempPeltSpIdle() { lock_guard lg(m_mtx); m_tempPeltSp.setIdle(); }

    void setTreatStarted(bool value) { lock_guard lg(m_mtx); m_treatmentStarted = value; }
    bool getTreatStarted() const { lock_guard lg(m_mtx); return m_treatmentStarted; }

    void setTreatStopped(bool value) { lock_guard lg(m_mtx); m_treatmentStopped = value; }
    bool getTreatStopped() const { lock_guard lg(m_mtx); return m_treatmentStopped; }

    void setDispBrightness(int value) { lock_guard lg(m_mtx); m_dispBrightness = value; }
    int getDispBrightness() const { lock_guard lg(m_mtx); return m_dispBrightness; }

    // clang-format on

private:
    float m_tempAir;
    PeltierTemperatures m_tempPelt;
    biometric::Data m_biom;
    SetPoint<float> m_tempPeltSp;
    bool m_treatmentStarted;
    bool m_treatmentStopped;
    int m_dispBrightness; // [0, 100] [%]



#if defined(PRJ_DEBUG) && 0 // this can be enabled to use a button at a NTC input as a debug button
#define ___i2c_btnState_available (1)
public:
    static constexpr float s_tempBtnState_threshold = 90.0f;

    // clang-format off
    bool getBtnState() const { lock_guard lg(m_mtx); return m_btnState; }
    void setBtnState(bool state) { lock_guard lg(m_mtx); m_btnState = state; }
    // clang-format on

private:
    bool m_btnState = 0;
#endif // PRJ_DEBUG
};

extern ThreadSharedData sd;

void thread();

} // namespace i2cThread


#endif // IG_MIDDLEWARE_I2CTHREAD_H

``````


### src/middleware/theme-light.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/middleware/theme-light.cpp
`relative_path`: src/middleware/theme-light.cpp
`format`: Arbitrary Binary Data
`size`: 12726   




### src/deviceabstraction/max32664defs.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/max32664defs.h
`relative_path`: src/deviceabstraction/max32664defs.h
`format`: Arbitrary Binary Data
`size`: 7226   




### src/deviceabstraction/mcp4728.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/mcp4728.cpp
`relative_path`: src/deviceabstraction/mcp4728.cpp
`format`: Arbitrary Binary Data
`size`: 1789   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cerrno>
#include <cstddef>
#include <cstdint>
#include <cstring>

#include "mcp4728.h"


#define LOG_MODULE_LEVEL LOG_LEVEL_DBG
#define LOG_MODULE_NAME  MCP4728
#include "middleware/log.h"


namespace {}



namespace deviceabstr::i2c {


MCP4728::MCP4728(const char* dev, uint8_t addr)
    : i2c_device(dev, addr)
{
    if (m_err) { LOG_ERR("failed to open %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno)); }
    else
    {
        // no init needed, is done in output method with multi-write command

        // test if device is on bus
        this->m_write(0);
        if (m_err) { LOG_ERR("device not on bus %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno)); }
    }
}

int MCP4728::output(uint8_t ch, uint16_t value)
{
    if (ch > 3)
    {
        LOG_ERR("invalid channel: %i", (int)ch);
        return -(__LINE__);
    }

    // constexpr uint8_t cmdFstWr = 0x00; // C0 don't care, write function not used (/UDAC bit is ignored)
    constexpr uint8_t cmdMltWr = 0x02;
    constexpr uint8_t wrfMltWr = 0x00;

    // mutli write command (fig. 5-8)
    m_buffer[0] = (cmdMltWr << 5) | (wrfMltWr << 3) | (ch << 1); // _UDAC = 0

    // VRef = 0 using Vdd (gain bit doesn't matter), PD = 0b00 normal use, high bits of value
    m_buffer[1] = (uint8_t)((value >> 8) & 0x000F);

    // low bits of value
    m_buffer[2] = (uint8_t)(value & 0x00FF);

    this->m_write(3);
    if (m_err)
    {
        LOG_ERR("failed to write output channel %i on %s addr 0x%02x - %i %s", (int)ch, m_dev.c_str(), m_addr, errno, std::strerror(errno));
        return m_err;
    }

    return 0;
}


} // namespace deviceabstr::i2c

``````


### src/deviceabstraction/ads1015.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/ads1015.h
`relative_path`: src/deviceabstraction/ads1015.h
`format`: Arbitrary Binary Data
`size`: 844   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_DEVICEABSTR_ADS1015_H
#define IG_DEVICEABSTR_ADS1015_H

#include <array>
#include <cstddef>
#include <cstdint>
#include <string>

#include "deviceabstraction/i2c-device.h"


namespace deviceabstr::i2c {

class ADS1015 : public i2c_device<3>
{
public:
    static constexpr uint8_t default_i2c_addr = 0x48;

public:
    ADS1015() = delete;

    explicit ADS1015(const char* dev, uint8_t addr = default_i2c_addr);

    virtual ~ADS1015() {}

    int readCh(uint8_t ch, uint16_t* value = nullptr);

    uint16_t getValue(uint8_t ch) const { return m_value[ch < 4 ? ch : 0]; }
    uint16_t getVolt_mV(uint8_t ch) const;

private:
    uint16_t m_value[4];
};

} // namespace deviceabstr::i2c


#endif // IG_DEVICEABSTR_ADS1015_H

``````


### src/deviceabstraction/ds1077l.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/ds1077l.cpp
`relative_path`: src/deviceabstraction/ds1077l.cpp
`format`: Arbitrary Binary Data
`size`: 5096   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cerrno>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <stdexcept>

#include "ds1077l.h"

#include <unistd.h>


#define LOG_MODULE_LEVEL LOG_LEVEL_INF
#define LOG_MODULE_NAME  DS1077L
#include "middleware/log.h"


#define CMD_DIV (0x01) // access DIV register
#define CMD_MUX (0x02) // access MUX register
#define CMD_BUS (0x0D) // access BUS register (also performs command E2 implicitly)
#define CMD_E2  (0x3F) // write register values to EEPROM

// bits regarding to out1 are 0 to make out1 OR configurable without changing out0 configuration
#define MUX_DEFAULT (0x0000) // PDN0=SEL0=EN0=0, out1 bits = 0


namespace {}



namespace deviceabstr::i2c {


DS1077L::DS1077L(const char* dev, uint8_t addr)
    : i2c_device(dev, addr)
{
    if (m_err) { LOG_ERR("failed to open %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno)); }
    else
    {
        try
        {
            // init MUX register
            m_writeMux(MUX_DEFAULT);
            if (m_err) { throw -(__LINE__); }
            m_awaitEeprom(); // actually only needed on the first boot, but doesn't hurt if done anyway

            // read BUS register
            m_buffer[0] = CMD_BUS;
            this->m_write(1);
            if (m_err) { throw std::runtime_error("failed to access BUS register"); }
            this->m_read(1);
            if (m_err) { throw std::runtime_error("failed to read BUS register"); }
            const uint8_t busRegister = m_buffer[0];

            if ((busRegister & 0x08) == 0) // first boot of software, WC bit is not yet set
            {
                LOG_INF("WC bit is cleared, setting it");

                // WC bit has to be set, so that not every change of a register value is going to be written to EEPROM

                // write BUS register with WC=1 and A=0x0
                m_buffer[0] = CMD_BUS;
                m_buffer[1] = 0x08;
                this->m_write(2);
                if (m_err) { throw std::runtime_error("failed to write BUS register"); }
                m_awaitEeprom();
            }
            else
            {
                LOG_DBG("WC bit is set");

#if defined(_DEBUG) && 0
                LOG_WRN("\n\nclearing WC bit\n");

                m_buffer[0] = CMD_BUS;
                m_buffer[1] = 0x00;
                this->write(2);
                if (m_err) { LOG_ERR("failed to write BUS register on %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno)); }
                m_awaitEeprom();
#endif
            }
        }
        catch (const std::exception& ex)
        {
            LOG_ERR("%s on %s addr 0x%02x - %i %s", ex.what(), m_dev.c_str(), m_addr, errno, std::strerror(errno));
        }
        catch (const int& intex)
        {
            LOG_DBG(LOG_SGR_BCYAN "catch int");
            // nop, some private function has already logged a message
            (void)intex;
        }
        catch (...)
        {
            const std::runtime_error ex("unknown exception");
            LOG_ERR("%s on %s addr 0x%02x - %i %s", ex.what(), m_dev.c_str(), m_addr, errno, std::strerror(errno));
        }
    }
}

int DS1077L::setOut0(uint8_t m0)
{
    // if this function is implemented, init of MUX register in ctor and DS1077L::setOut1() might need adaptions
    throw std::runtime_error("DS1077L::setOut0() not implemented");
}

int DS1077L::setOut1(uint8_t m1, uint8_t div1, uint16_t n1)
{
    const int errM = m_writeMux(MUX_DEFAULT | ((m1 & 0x03) << 7) | ((div1 & 0x01) << 6));

    const int errN = m_writeDiv(n1 << 6);

    if (errM || errN) { return -(__LINE__); }
    return 0;
}

int DS1077L::m_writeMux(uint16_t value)
{
    m_buffer[0] = CMD_MUX;
    m_buffer[1] = (value >> 8) & 0xFF;
    m_buffer[2] = value & 0xFF;

    this->m_write(3);

    if (m_err) { LOG_ERR("failed to write MUX register on %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno)); }

    return m_err;
}

int DS1077L::m_writeDiv(uint16_t value)
{
    m_buffer[0] = CMD_DIV;
    m_buffer[1] = (value >> 8) & 0xFF;
    m_buffer[2] = value & 0xFF;

    this->m_write(3);

    if (m_err) { LOG_ERR("failed to write DIV register on %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno)); }

    return m_err;
}

int DS1077L::m_awaitEeprom()
{
    constexpr int wait_us = 20;
    constexpr int timeout_ms = 100;

#if defined(_DEBUG) && 0
    constexpr int maxCnt = 2;
    (void)timeout_ms;
#else
    constexpr int maxCnt = timeout_ms * 1000 / wait_us;
#endif

    int cnt = 0;

    do {
        // while the EEPROM is being written the device responds with NAK, which setts m_err
        this->m_write(0);

        usleep(wait_us);

        ++cnt;

        LOG_DBG("eeprom delay loop cnt: %i/%i", cnt, maxCnt);

        if (m_err && (cnt >= maxCnt))
        {
            LOG_WRN("EEPROM timeout");
            return -(__LINE__);
        }
    }
    while (m_err);

    return 0;
}



} // namespace deviceabstr::i2c

``````


### src/deviceabstraction/max32664.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/max32664.h
`relative_path`: src/deviceabstraction/max32664.h
`format`: Arbitrary Binary Data
`size`: 3437   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_DEVICEABSTR_MAX32664_H
#define IG_DEVICEABSTR_MAX32664_H

#include <array>
#include <cstddef>
#include <cstdint>
#include <string>

#include "deviceabstraction/i2c-device.h"


#define MAX32664_DATA_BUFFER_SIZE (1024)


namespace deviceabstr::i2c {

class MAX32664 : public i2c_device<MAX32664_DATA_BUFFER_SIZE + 1>
{
public:
    static constexpr uint8_t default_i2c_addr = 0x55;

    /**
     * @brief Returns the CMD_DELAY as ms for the requested command.
     */
    static uint32_t cmdDelay(uint8_t family, uint8_t index, uint8_t writeByte0);

    // static void parse();

public:
    MAX32664() = delete;

    MAX32664(int gpioPin_mfio, int gpioPin_nRst, const char* dev, uint8_t addr = default_i2c_addr);

    virtual ~MAX32664() {}

    void config_MFIO_in();
    void config_MFIO_out();

    bool readPin_MFIO() const;
    void writePin_MFIO(bool state);
    void writePin_nRST(bool state);



public:
    //==================================================================================================================
    // generic transactions
    //
    // A I2C transaction is always a write and a read (see "Figure 10", "I2C Write" and "I2C Read" in UG6806; Rev 3; 8/20):
    // 1. start transaction with I2C write
    // 2. wait command delay
    // 3. end transaction with I2C read (read status byte + optional data)

    uint8_t transactionRSB() const { return m_readStatusByte; }
    bool transactionOk() const { return (this->good() && (m_readStatusByte == 0x00 /* READ_STATUS_OK */)); }



    /**
     * @brief Starts a transaction.
     *
     * The start of a tranaction is the same for read an write transactions.
     *
     * See comment about "generic transactions" above.
     *
     * Side effects: `m_readStatusByte` is set to `0xFF`, and `m_err` may get set.
     *
     * @param family Family byte
     * @param index Index byte
     * @param data Optional data
     * @param count Number of data bytes
     * @return 0 on success
     */
    int transaction(uint8_t family, uint8_t index, const uint8_t* data = nullptr, size_t count = 0);
    int transaction(uint8_t family, uint8_t index, uint8_t writeByte0) { return transaction(family, index, &writeByte0, 1); }

    /**
     * @brief Finalises a transaction by reading the status byte.
     *
     * See comment about "generic transactions" above.
     *
     * Side effects: `m_err` may get set, if so, `m_readStatusByte` is set to `0xFF`.
     *
     * @return 0 on success
     */
    int transactionEnd();

    /**
     * @brief Finalises a read transaction by reading the status byte.
     *
     * See comment about "generic transactions" above.
     *
     * Side effects: `m_err` may get set, if so, `m_readStatusByte` is set to `0xFF`.
     *
     * @param buffer Pointer to the buffer where the read data (not including the read status byte) is written to
     * @param count Number of data bytes to be read
     * @return 0 on success
     */
    int transactionReadEnd(uint8_t* buffer, size_t count);

    // end of generic transactions
    //==================================================================================================================



private:
    int m_pin_mfio;
    int m_pin_nRst;
    uint8_t m_readStatusByte;
};

} // namespace deviceabstr::i2c


#endif // IG_DEVICEABSTR_MAX32664_H

``````


### src/deviceabstraction/mcp9808.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/mcp9808.h
`relative_path`: src/deviceabstraction/mcp9808.h
`format`: Arbitrary Binary Data
`size`: 745   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_DEVICEABSTR_MCP9808_H
#define IG_DEVICEABSTR_MCP9808_H

#include <array>
#include <cstddef>
#include <cstdint>
#include <string>

#include "deviceabstraction/i2c-device.h"


namespace deviceabstr::i2c {

class MCP9808 : public i2c_device<3>
{
public:
    static constexpr uint8_t default_i2c_addr = 0x18;

public:
    MCP9808() = delete;

    explicit MCP9808(const char* dev, uint8_t addr = default_i2c_addr);

    virtual ~MCP9808() {}

    int readTemp(float* p = nullptr);

    float getTemp() const { return m_temp; }

private:
    float m_temp;
};

} // namespace deviceabstr::i2c


#endif // IG_DEVICEABSTR_MCP9808_H

``````


### src/deviceabstraction/mcp4728.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/mcp4728.h
`relative_path`: src/deviceabstraction/mcp4728.h
`format`: Arbitrary Binary Data
`size`: 971   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_DEVICEABSTR_MCP4728_H
#define IG_DEVICEABSTR_MCP4728_H

#include <array>
#include <cstddef>
#include <cstdint>
#include <string>

#include "deviceabstraction/i2c-device.h"


namespace deviceabstr::i2c {

class MCP4728 : public i2c_device<8>
{
public:
    static constexpr uint8_t i2c_addr_min = 0x60;
    static constexpr uint8_t i2c_addr_max = 0x67;
    static constexpr uint8_t default_i2c_addr = i2c_addr_min;

public:
    MCP4728() = delete;

    explicit MCP4728(const char* dev, uint8_t addr = default_i2c_addr);

    virtual ~MCP4728() {}

    // int write(uint8_t ch, uint16_t value);
    // int writeAll(uint16_t a, uint16_t b, uint16_t c, uint16_t d);

    int output(uint8_t ch, uint16_t value);
    // int outputAll(uint16_t a, uint16_t b, uint16_t c, uint16_t d);
};

} // namespace deviceabstr::i2c


#endif // IG_DEVICEABSTR_MCP4728_H

``````


### src/deviceabstraction/max32664.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/max32664.cpp
`relative_path`: src/deviceabstraction/max32664.cpp
`format`: Arbitrary Binary Data
`size`: 7501   




### src/deviceabstraction/i2c-device.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/i2c-device.h
`relative_path`: src/deviceabstraction/i2c-device.h
`format`: Arbitrary Binary Data
`size`: 5543   




### src/deviceabstraction/ads1015.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/ads1015.cpp
`relative_path`: src/deviceabstraction/ads1015.cpp
`format`: Arbitrary Binary Data
`size`: 4575   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cerrno>
#include <cstddef>
#include <cstdint>
#include <cstring>

#include "ads1015.h"

#include <unistd.h>


#define LOG_MODULE_LEVEL LOG_LEVEL_DBG
#define LOG_MODULE_NAME  ADS1015
#include "middleware/log.h"


#define CONFIG_HI_SRARTCONV   (0x80)
#define CONFIG_HI_MUX_SE      (0x40) // single ended, bit4:5 (mask 0x30) specify the channel
#define CONFIG_HI_PGA_6144    (0x00 << 1u)
#define CONFIG_HI_PGA_4096    (0x01 << 1u)
#define CONFIG_HI_PGA_2048    (0x02 << 1u)
#define CONFIG_HI_PGA_1024    (0x03 << 1u)
#define CONFIG_HI_PGA_0512    (0x04 << 1u)
#define CONFIG_HI_PGA_0256    (0x05 << 1u)
// #define CONFIG_HI_PGA_0256  (0x06 << 1u)
// #define CONFIG_HI_PGA_0256  (0x07 << 1u)
#define CONFIG_HI_MODE_CONT   (0x00)
#define CONFIG_HI_MODE_SINGLE (0x01)
#define CONFIG_LO_DR_128      (0x00 << 5u)
#define CONFIG_LO_DR_250      (0x01 << 5u)
#define CONFIG_LO_DR_490      (0x02 << 5u)
#define CONFIG_LO_DR_920      (0x03 << 5u)
#define CONFIG_LO_DR_1600     (0x04 << 5u)
#define CONFIG_LO_DR_2400     (0x05 << 5u)
#define CONFIG_LO_DR_3300     (0x06 << 5u)
#define CONFIG_LO_COMP_DIS    (0x03) // disable comparator output

namespace {

constexpr uint8_t configHiPga = CONFIG_HI_PGA_4096;
constexpr uint16_t lsb_mV = 2 /* because +/-FS */ * 4096 /* Vref' */ / 4096 /* 2^12 */;
// constexpr float lsb_mV = 2.0f * 512.0f / /* 2^12 */ 4096.0f;

}



namespace deviceabstr::i2c {


ADS1015::ADS1015(const char* dev, uint8_t addr)
    : i2c_device(dev, addr), m_value{ 0, 0, 0, 0 }
{
    if (m_err) { LOG_ERR("failed to open %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno)); }
    else
    {
        // no init needed, device is in single-shot/power-down mode by default and config register has to be written
        // before every conversion anyway

        // test if device is on bus
        this->m_write(0);
        if (m_err) { LOG_ERR("device not on bus %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno)); }
    }
}

int ADS1015::readCh(uint8_t ch, uint16_t* value)
{
    if (ch > 3)
    {
        LOG_ERR("invalid channel: %i", (int)ch);
        return -(__LINE__);
    }

    uint16_t* const pValue = m_value + ch;

    // write config register
    {
        m_buffer[0] = 0x01; // config register

        m_buffer[1] = CONFIG_HI_SRARTCONV | CONFIG_HI_MUX_SE | (ch << 4u) | configHiPga | CONFIG_HI_MODE_SINGLE;
        m_buffer[2] = CONFIG_LO_DR_3300 | CONFIG_LO_COMP_DIS;

        // LOG_INF("0x%02x%02x", m_buffer[1], m_buffer[2]);

        this->m_write(3);
        if (m_err)
        {
            LOG_ERR("failed to config input channel %i on %s addr 0x%02x - %i %s", (int)ch, m_dev.c_str(), m_addr, errno, std::strerror(errno));
            return m_err;
        }
    }



    // usleep(303); // 1/3300sps = 303.03us     ads1015
    // usleep(1163); // 1/860sps = 1162.79us    ads1115

#warning "using timing of the ADS1115 anyway, which is OK in this projects software structure"

    // From the datasheet: Conversions in the ADS111x settle within a single cycle; thus, the conversion time is equal to 1 / DR.
    // This is not true! It is significantly more, even with factor 1.5 channels get mixed up.
    usleep(1163 * 2); // ads1115 timing is also OK for ads1015 (it is faster)



    // read conv result register
    {
        m_buffer[0] = 0x00; // conversation register
        this->m_write(1);
        if (m_err)
        {
            LOG_ERR("failed to set register pointer on %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno));
            return m_err;
        }

        this->m_read(2);
        if (m_err)
        {
            LOG_ERR("failed to read conversation register on %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno));
            return m_err;
        }

        *pValue = m_buffer[0];
        *pValue <<= 8;
        *pValue |= m_buffer[1];

        const bool isNegative = ((*pValue & 0x8000) ? true : false);

        // right align 12bit result
        *pValue >>= 4;

#if 1 // clamp to 0

        if (isNegative) { *pValue = 0; }

#else // real result

        // right shift is not striclty defined in C/C++
        if (isNegative) { *pValue |= 0xF000; }
        else { *pValue &= ~0xF000; }
#endif
    }



    if (value) { *value = *pValue; }

    return 0;
}

uint16_t ADS1015::getVolt_mV(uint8_t ch) const { return (uint16_t)(this->getValue(ch) * lsb_mV); }



} // namespace deviceabstr::i2c

``````


### src/deviceabstraction/mcp9808.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/mcp9808.cpp
`relative_path`: src/deviceabstraction/mcp9808.cpp
`format`: Arbitrary Binary Data
`size`: 1793   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cerrno>
#include <cstddef>
#include <cstdint>
#include <cstring>

#include "mcp9808.h"


#define LOG_MODULE_LEVEL LOG_LEVEL_DBG
#define LOG_MODULE_NAME  MCP9808
#include "middleware/log.h"


namespace {}



namespace deviceabstr::i2c {


MCP9808::MCP9808(const char* dev, uint8_t addr)
    : i2c_device(dev, addr), m_temp(0)
{
    if (m_err) { LOG_ERR("failed to open %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno)); }
    else
    {
        m_buffer[0] = 0x08; // resolution register
        m_buffer[1] = 0x01; // 0.25degC t_CONV = 65ms typical
        this->m_write(2);

        if (m_err) { LOG_ERR("failed to config resolution on %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno)); }
    }
}

int MCP9808::readTemp(float* p)
{
    m_buffer[0] = 0x05; // temperature register
    this->m_write(1);
    if (m_err)
    {
        LOG_ERR("failed to set register pointer on %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno));
        return m_err;
    }

    this->m_read(2);
    if (m_err)
    {
        LOG_ERR("failed to read temperature register on %s addr 0x%02x - %i %s", m_dev.c_str(), m_addr, errno, std::strerror(errno));
        return m_err;
    }

    uint16_t reg; // register value

    reg = m_buffer[0];
    reg <<= 8;
    reg |= m_buffer[1];

    const bool isNegative = ((reg & 0x1000) ? true : false);

    // EQUATION 5-1 on page 25 in the datasheet
    m_temp = (float)(reg & 0x0FFF) / 16.0f;
    if (isNegative) { m_temp = 256.0f - m_temp; }
#warning "not tested for negative temperatures"

    if (p) { *p = m_temp; }

    return 0;
}


} // namespace deviceabstr::i2c

``````


### src/deviceabstraction/ds1077l.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/deviceabstraction/ds1077l.h
`relative_path`: src/deviceabstraction/ds1077l.h
`format`: Arbitrary Binary Data
`size`: 1408   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_DEVICEABSTR_DS1077L_H
#define IG_DEVICEABSTR_DS1077L_H

#include <array>
#include <cstddef>
#include <cstdint>
#include <string>

#include "deviceabstraction/i2c-device.h"


namespace deviceabstr::i2c {

class DS1077L : public i2c_device<3> // command + MSByte + LSByte = 3
{
public:
    static constexpr uint8_t default_i2c_addr = 0x58;

public:
    DS1077L() = delete;

    explicit DS1077L(const char* dev, uint8_t addr = default_i2c_addr);

    virtual ~DS1077L() {}

    int setOut0(uint8_t m0);

    /**
     * @brief Set the frequency of OUT1
     *
     * f_out = MCLK * 1/(2^M) * 1/(N + 2)   if DIV1=0
     * f_out = MCLK * 1/(2^M)               if DIV1=1
     *
     * @param m1 Prescaler, range: [0, 3]
     * @param div1 Bypass N divider, range: [0, 1]
     * @param n1 N divider, range: [0, 1023] => [0x000, 0x3FF]
     * @return 0 on success
     */
    int setOut1(uint8_t m1, uint8_t div1, uint16_t n1);

private:
    int m_writeMux(uint16_t value);
    int m_writeDiv(uint16_t value);

    /**
     * @brief Waits for EEPROM write to be done.
     *
     * This is waiting while NAK.
     *
     * @return 0 if EEPROM write done (ACK), negative if NAK and timeout
     */
    int m_awaitEeprom();
};

} // namespace deviceabstr::i2c


#endif // IG_DEVICEABSTR_DS1077L_H

``````


### src/api/api.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/api/api.cpp
`relative_path`: src/api/api.cpp
`format`: Arbitrary Binary Data
`size`: 30467   




### src/api/data.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/api/data.h
`relative_path`: src/api/data.h
`format`: Arbitrary Binary Data
`size`: 4033   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_API_DATA_H
#define IG_API_DATA_H

#include <cstddef>
#include <cstdint>
#include <string>

#include "application/treatmentData.h"


namespace api {

class req_data_base
{
public:
    req_data_base() {}
    virtual ~req_data_base() {}

    // aka serialise
    virtual std::string httpBody() const = 0;
};

class res_data_base
{
public:
    res_data_base() {}
    virtual ~res_data_base() {}

    virtual void set(const std::string& httpBody) noexcept(false) = 0;
};



//======================================================================================================================
// requests

class CommissionReq : public req_data_base // TODO is a derived class really needed? comission id is already in appdata in the default req JSON
{
public:
    class ID
    {
    public:
        ID() = delete;

        explicit ID(const std::string& id)
            : m_id(id)
        {}

        const std::string& get() const { return m_id; }

    private:
        std::string m_id;
    };

public:
    CommissionReq()
        : req_data_base(), m_id()
    {}

    CommissionReq(const ID& id)
        : req_data_base(), m_id(id.get())
    {}

    virtual ~CommissionReq() {}

    virtual std::string httpBody() const;

private:
    std::string m_id;
};

class StartReq : public req_data_base
{
public:
    StartReq()
        : req_data_base()
    {}

    virtual ~StartReq() {}

    virtual std::string httpBody() const;
};

class SettingsReq : public req_data_base
{
public:
    SettingsReq()
        : req_data_base()
    {}

    virtual ~SettingsReq() {}

    virtual std::string httpBody() const;
};

class ProgressReq : public req_data_base
{
public:
    ProgressReq()
        : req_data_base()
    {}

    virtual ~ProgressReq() {}

    virtual std::string httpBody() const;
};

class ErrorReq : public req_data_base
{
public:
    ErrorReq()
        : req_data_base()
    {}

    virtual ~ErrorReq() {}

    virtual std::string httpBody() const;
};



//======================================================================================================================
// responses

class CommissionRes : public res_data_base
{
public:
    CommissionRes()
        : res_data_base(), m_dataReady(false), m_machineId(), m_timezone(), m_wifiCountry()
    {}

    virtual ~CommissionRes() {}

    bool dataReady() const { return m_dataReady; }

    const std::string& machineId() const { return m_machineId; }
    const std::string& timezone() const { return m_timezone; }
    const std::string& wifiCountry() const { return m_wifiCountry; }

    virtual void set(const std::string& httpBody) noexcept(false);

private:
    bool m_dataReady;
    std::string m_machineId;
    std::string m_timezone;
    std::string m_wifiCountry;
};

class StartRes : public res_data_base
{
public:
    StartRes()
        : res_data_base(), m_treatClearance(false)
    {}

    virtual ~StartRes() {}

    bool treatClearance() const { return m_treatClearance; }

    virtual void set(const std::string& httpBody) noexcept(false);

private:
    bool m_treatClearance;
};

class SettingsRes : public res_data_base
{
public:
    SettingsRes()
        : res_data_base(), m_treatConfig(app::treat::Config::blocks_type(), "#FFFFFF"), m_user("<nickname>")
    {}

    virtual ~SettingsRes() {}

    const app::treat::User& user() const { return m_user; }
    const app::treat::Config& treatConfig() const { return m_treatConfig; }

    virtual void set(const std::string& httpBody) noexcept(false);

private:
    app::treat::Config m_treatConfig;
    app::treat::User m_user;
};

class ProgressRes : public res_data_base
{
public:
    ProgressRes()
        : res_data_base(), m_abort(false)
    {}

    virtual ~ProgressRes() {}

    bool abortTreatment() const { return m_abort; }

    virtual void set(const std::string& httpBody) noexcept(false);

private:
    bool m_abort;
};

} // namespace api


#endif // IG_API_DATA_H

``````


### src/api/data.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/api/data.cpp
`relative_path`: src/api/data.cpp
`format`: Arbitrary Binary Data
`size`: 8417   




### src/api/api.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/api/api.h
`relative_path`: src/api/api.h
`format`: Arbitrary Binary Data
`size`: 4573   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

/*

data:
 - set req data (one per endpoint)
 - get res data (one per endpoint)
 - status
    - state (idle, req, res, error)
    - current endpoint (don't care in idle state)
    - timestamps (req, res)
    - duration

Serialising of the req data and parsing of the res data is explicitly done in the API thread.

controlling / thread sync:
 - set req data triggers the request, returns with no effect if state != idle
 - get res data sets state to idle
 - flush method to set state to idle (in case of error), returns with no effect if state is neither error nor res

*/

#ifndef IG_API_API_H
#define IG_API_API_H

#include <cstddef>
#include <cstdint>
#include <ctime>
#include <string>

#include "api/data.h"
#include "middleware/thread.h"
#include "middleware/util.h"


namespace api::thread {

typedef enum
{
    ep_commission,
    ep_start,    // start treatment clearance
    ep_settings, // treatment/user config
    ep_progress, // report treatment progress/data
    ep_error,

    ep__end_
} endpoint_t;

typedef enum STATE
{
    state_idle,  // ready to make a request
    state_req,   // request is ongoing
    state_res,   // response is ready
    state_error, // an error occured

    state__end_
} state_t;

class Status
{
public:
    Status()
        : m_state(state_idle), m_ep(), m_tReq(), m_tRes(), m_dur(0)
    {}

    virtual ~Status() {}

    int reqEndpoint(endpoint_t ep);
    void resGotten(endpoint_t ep);
    void setState(state_t state) { m_state = state; }
    void setTReq(time_t t) { m_tReq = t; }
    void setTRes(time_t t) { m_tRes = t; }
    void setDuration(omw_::clock::timepoint_t dur_us) { m_dur = dur_us; }

    state_t state() const { return m_state; }
    endpoint_t endpoint() const { return m_ep; }
    time_t tReq() const { return m_tReq; }
    time_t tRes() const { return m_tRes; }
    omw_::clock::timepoint_t duration() const { return m_dur; }

private:
    state_t m_state;
    endpoint_t m_ep;
    time_t m_tReq;
    time_t m_tRes;
    omw_::clock::timepoint_t m_dur;
};



std::string toString(endpoint_t ep);



class ThreadSharedData : public ::thread::SharedData
{
public:
    ThreadSharedData()
        : m_status()
    {}

    virtual ~ThreadSharedData() {}



    // extern
public:
    int reqCommission(const CommissionReq& data);
    int reqStart(const StartReq& data);
    int reqSettings(const SettingsReq& data);
    int reqProgress(const ProgressReq& data);
    int reqError(const ErrorReq& data);

    CommissionRes getCommissionRes() const;
    StartRes getStartRes() const;
    SettingsRes getSettingsRes() const;
    ProgressRes getProgressRes() const;

    void flush() const;

    // clang-format off
    Status status() const { lock_guard lg(m_mtx); return m_status; }
    // clang-format on



    // intern
public:
    // clang-format off
    void setState(state_t state)    { lock_guard lg(m_mtx); m_status.setState(state); }
    void setTReq(time_t t)          { lock_guard lg(m_mtx); m_status.setTReq(t); }
    void setTRes(time_t t)          { lock_guard lg(m_mtx); m_status.setTRes(t); }
    void setDur(omw_::clock::timepoint_t dur_us) { lock_guard lg(m_mtx); m_status.setDuration(dur_us); }

    void setCommissionRes(const CommissionRes& data) { lock_guard lg(m_mtx); m_commissionRes = data; }
    void setStartRes(const StartRes& data) { lock_guard lg(m_mtx); m_startRes = data; }
    void setSettingsRes(const SettingsRes& data) { lock_guard lg(m_mtx); m_settingsRes = data; }
    void setProgressRes(const ProgressRes& data) { lock_guard lg(m_mtx); m_progressRes = data; }

    CommissionReq getCommissionReq() const { lock_guard lg(m_mtx); return m_commissionReq; }
    StartReq getStartReq() const { lock_guard lg(m_mtx); return m_startReq; }
    SettingsReq getSettingsReq() const { lock_guard lg(m_mtx); return m_settingsReq; }
    ProgressReq getProgressReq() const { lock_guard lg(m_mtx); return m_progressReq; }
    ErrorReq getErrorReq() const { lock_guard lg(m_mtx); return m_errorReq; }
    // clang-format on



private:
    mutable Status m_status;
    CommissionReq m_commissionReq;
    CommissionRes m_commissionRes;
    StartReq m_startReq;
    StartRes m_startRes;
    SettingsReq m_settingsReq;
    SettingsRes m_settingsRes;
    ProgressReq m_progressReq;
    ProgressRes m_progressRes;
    ErrorReq m_errorReq;
};

extern ThreadSharedData sd;

void thread();

time_t getReqInterval(time_t min, time_t max);

} // namespace api::thread


#endif // IG_API_API_H

``````


### src/project.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/project.h
`relative_path`: src/project.h
`format`: Arbitrary Binary Data
`size`: 1577   


``````
/*
author          Oliver Blaser
date            22.05.2024
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_PROJECT_H
#define IG_PROJECT_H

#include <omw/defs.h>
#include <omw/version.h>


#define PRJ_ENTER_LOCALSETUP_FROM_IDLE (1)
#define PRJ_EN_PREPARE_STATE           (0)
#define PRJ_USE_HTML_IO_EMU            (0)
#define PRJ_START_AT_HANDS_ON          (0)
#define PRJ_EN_CONSOLE_STATUS_LINE     (1)
#define PRJ_PLAIN_RASPI                (0) // see /build/readme.md

#define PRJ_PKG_DEV_DISPDMY (0)


#if PRJ_PKG_DEV_DISPDMY
#define PRJ_VERSION_BUILD ("DEV-DISPDMY")
#elif defined(_DEBUG)
#define PRJ_VERSION_BUILD ("DEBUG")
#endif // PRJ_PKG_DEV_DISPDMY

namespace prj {

const char* const appDirName = "vacuul"; // eq to package name

const char* const appName = "Vacuul";
const char* const binName = "vacuul"; // eq to the linker setting


#ifndef PRJ_VERSION_BUILD
#define PRJ_VERSION_BUILD ("")
#endif
const omw::Version version(0, 3, 2, "alpha", PRJ_VERSION_BUILD);

// const char* const website = "https://silab.ch/";

} // namespace prj


#ifdef OMW_DEBUG
#define PRJ_DEBUG (1)
#else
#undef PRJ_DEBUG
#endif



#if PRJ_PKG_DEV_DISPDMY && !PRJ_PLAIN_RASPI
#warning "overriding PRJ_PLAIN_RASPI"
#undef PRJ_PLAIN_RASPI
#define PRJ_PLAIN_RASPI (1)
#endif



#ifndef RPIHAL_EMU

#ifndef __linux__
#error "not a Linux platform"
#endif

#if (!defined(__arm__) && !defined(__aarch64__))
#error "not an ARM platform" // nothing really ARM speciffic is used, just to detect RasPi
#endif

#endif // RPIHAL_EMU


#endif // IG_PROJECT_H

``````


### src/application/treatment.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/treatment.cpp
`relative_path`: src/application/treatment.cpp
`format`: Arbitrary Binary Data
`size`: 15811   




### src/application/commissioning.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/commissioning.cpp
`relative_path`: src/application/commissioning.cpp
`format`: Arbitrary Binary Data
`size`: 4517   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ctime>

#include "api/api.h"
#include "application/config.h"
#include "application/status.h"
#include "commissioning.h"

#include <omw/string.h>
#include <unistd.h>


#define LOG_MODULE_LEVEL LOG_LEVEL_DBG
#define LOG_MODULE_NAME  COMIS
#include "middleware/log.h"


namespace {

enum
{
    S_init = 0,
    S_idle,
    S_req,
    S_awaitRes,
    S_process,
    S_retry,
    S_done,
};

std::string generateCommissionId()
{
    std::string str;

    // const char* const vinDigits = "1234567890ABCDEFGHJKLMNPRSTUVWXYZ"; // VIN - vehicle identification number
    static const char digits[] = "1234567890ABCDEFGHJKLMNPRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    constexpr size_t digitsSize = SIZEOF_ARRAY(digits) - 1; // -1 because \0

    constexpr size_t digitsPerGrp = 4;
    constexpr size_t nGrps = 4;
    constexpr size_t len = (digitsPerGrp * nGrps);

    for (size_t i = 0; i < len; ++i)
    {
        if (((i % digitsPerGrp) == 0) && (i > 0)) { str += '-'; }

        str += *(digits + (rand() % digitsSize));
    }

    return str;
}

} // namespace



int app::commissioning::task()
{
    int r = commissioning::running;

    static int state = S_init;
    static std::string commissionId = "";
    static api::CommissionRes resData;
    static time_t tAttempt = 0;
    static size_t attemptCnt = 0;
    static time_t retryDelay = 0;

    const time_t tNow = time(nullptr);

    static int oldState = -1;
    if (oldState != state)
    {
        LOG_DBG("%s state %i -> %i", ___LOG_STR(LOG_MODULE_NAME), oldState, state);

        oldState = state;
    }

    switch (state)
    {
    case S_init:
        srand(tNow);
        commissionId = generateCommissionId();
        tAttempt = 0;
        retryDelay = 0;
        state = S_idle;
        break;

    case S_idle:
        if ((tNow - tAttempt) >= retryDelay)
        {
            tAttempt = tNow;
            state = S_req;
        }
        break;

    case S_req:
        if (api::thread::sd.reqCommission(api::CommissionReq::ID(commissionId) // TODO use real commission data class
                                          ) == 0)
        {
            LOG_DBG("req with commission ID: %s", commissionId.c_str());

            ++attemptCnt;
            app::appData.set(app::CommissionData(commissionId, attemptCnt));

            state = S_awaitRes;
        }
        break;

    case S_awaitRes:
    {
        const auto status = api::thread::sd.status();

        if (status.state() == api::thread::state_res)
        {
            resData = api::thread::sd.getCommissionRes();
            state = S_process;
        }
        else if (status.state() == api::thread::state_error)
        {
            api::thread::sd.flush();
            state = S_retry;
        }
    }
    break;

    case S_process:
        if (resData.dataReady())
        {
            const std::string machineId = resData.machineId();
            const std::string timezone = resData.timezone();
            const std::string wifiCountry = resData.wifiCountry();

            app::status.setMachineId(machineId);

            app::config.sys_wifiCountry = wifiCountry; // no more to do with the WiFi country (will be preset in local setup screen)
            app::config.sys_timezone = timezone;
            app::config.sys_machineId = machineId;
            const int err = app::config.save();
            if (err)
            {
                LOG_ERR("failed to save config file (%i)", err);
                status.addDbgMsg("failed to save machine ID to config file (" + std::to_string(err) + ")");

                return commissioning::error;
            }

            // TODO set timezone in system (currently not needed, the time is nowhere displayed as "wallclock")

            LOG_INF("commissioning OK - machine ID: %s, timezone: %s", machineId.c_str(), timezone.c_str());

            state = S_done;
        }
        else
        {
            LOG_DBG("commission data not ready");
            state = S_retry;
        }
        break;

    case S_retry:
        retryDelay = api::thread::getReqInterval(3, 8);
        LOG_DBG("retryDelay: %is", (int)retryDelay);
        state = S_idle;
        break;

    case S_done:
        state = S_init;
        r = commissioning::done;
        break;

    default:
        LOG_invalidState(state, 5);
        break;
    }

    return r;
}

``````


### src/application/status.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/status.h
`relative_path`: src/application/status.h
`format`: Arbitrary Binary Data
`size`: 5548   




### src/application/config.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/config.h
`relative_path`: src/application/config.h
`format`: Arbitrary Binary Data
`size`: 1798   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_CONFIG_H
#define IG_APP_CONFIG_H

#include <cstddef>
#include <cstdint>
#include <ctime>
#include <string>
#include <string_view>
#include <vector>

#include "middleware/iniFile.h"
#include "project.h"

#include <omw/string.h>
#include <omw/version.h>


namespace app {

class Config
{
public:
    static constexpr std::string_view default_cfg_binVer = "0.0.0";
    static constexpr bool default_cfg_writeCfgFile = true;

    static constexpr std::string_view default_sys_machineId = "0";
    static constexpr int default_sys_displayBrightness = 100;
    static constexpr std::string_view default_sys_wifiCountry = ""; // empty by design
    static constexpr std::string_view default_sys_timezone = "Europe/Berlin";

    // static constexpr bool default_ls_done = false;
    // static constexpr std::string_view default_ls_state = "-";

    static constexpr std::string_view default_api_baseUrl = "https://example.com/api/v1/"; // TODO add default API host

public:
    Config();

    omw::Version cfg_binVer;
    bool cfg_writeCfgFile;

    std::string sys_machineId;
    int sys_displayBrightness;
    std::string sys_wifiCountry;
    std::string sys_timezone;

    // bool ls_done;
    // std::string ls_state;

    std::string api_baseUrl;

    void setFileName(const std::string& name) { iniFile.setFileName(name); }
    const std::string& getFileName() const { return iniFile.getFileName(); }

    int getUpdateResult() const;

    int save();
    int read();

    std::string dump() const { return iniFile.dump(); }

private:
    mw::IniFile iniFile;
    int updateResult;

    int update();
};

extern Config config;

} // namespace app


#endif // IG_APP_CONFIG_H

``````


### src/application/treatmentData.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/treatmentData.h
`relative_path`: src/application/treatmentData.h
`format`: Arbitrary Binary Data
`size`: 2028   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_TREATMENTDATA_H
#define IG_APP_TREATMENTDATA_H

#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

#include <omw/color.h>



namespace app::treat {

constexpr float defaultTreatmentStartTemp = 17.0f; // this temperature is held in idle

class Block
{
public:
    enum class Type
    {
        treat = 0,
        pause,
        end,
    };

public:
    Block() = delete;

    // Block(Block::Type type, uint32_t dur_s)
    //     : m_type(type), m_temp(defaultTreatmentStartTemp), m_dur(dur_s)
    //{}

    Block(Block::Type type, uint32_t dur_s, float temp)
        : m_type(type), m_temp(temp), m_dur(dur_s)
    {}

    virtual ~Block() {}

    Block::Type type() const { return m_type; }

    float temp() const { return m_temp; }

    // block duration [s]
    uint32_t duration() const { return m_dur; }

private:
    Block::Type m_type;
    float m_temp;   // peltier temperature [degC]
    uint32_t m_dur; // duration [s]
};

extern const Block endBlock;

class Config
{
public:
    using blocks_type = std::vector<app::treat::Block>;

public:
    Config() = delete; // explicitly init empty config

    Config(const blocks_type& blocks, const omw::Color& ledCol)
        : m_blocks(blocks), m_ledColour(ledCol)
    {}

    virtual ~Config() {}

    const Block& getBlock(blocks_type::size_type idx) const;
    const omw::Color& getLedColour() const { return m_ledColour; }

    // treatment duration [s]
    uint32_t duration() const;

private:
    blocks_type m_blocks;
    omw::Color m_ledColour;
};

class User
{
public:
    User(const std::string& nickname)
        : m_nickname(nickname)
    {}

    virtual ~User() {}

    const std::string& nickname() const { return m_nickname; }

private:
    std::string m_nickname;
    // user id (needed if offline logging (to RAM!) would be a thing in future)
};

} // namespace app::treat


#endif // IG_APP_TREATMENTDATA_H

``````


### src/application/status.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/status.cpp
`relative_path`: src/application/status.cpp
`format`: Arbitrary Binary Data
`size`: 3993   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>

#include "middleware/i2cThread.h"
#include "middleware/json.h"
#include "middleware/otp.h"
#include "project.h"
#include "status.h"


#define LOG_MODULE_LEVEL LOG_LEVEL_INF
#define LOG_MODULE_NAME  STATUS
#include "middleware/log.h"


namespace {}


namespace app {

void AppData::set(const CommissionData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_commissionData = data;
    m_procData = &m_commissionData;
}

void AppData::set(const LocalSetupData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_localSetupData = data;
    m_procData = &m_localSetupData;
}

#ifdef ___CLASS_PrepareData_DECLARED
void AppData::set(const PrepareData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_prepareData = data;
    m_procData = &m_prepareData;
}
#endif // ___CLASS_PrepareData_DECLARED

void AppData::set(const IdleData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_idleData = data;
    m_procData = &m_idleData;
}

void AppData::set(const TreatmentData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_treatmentData = data;
    m_procData = &m_treatmentData;
}

void AppData::set(const ErrorData& data)
{
    lock_guard lg(m_mtx);

    ++m_updateIdx;

    m_errorData = data;
    m_procData = &m_errorData;
}

bool AppData::handsOn() const
{
    bool r = false;

    if (&m_treatmentData == m_procData) { r = (static_cast<const TreatmentData*>(m_procData))->handsOn(); }

    return r;
}

std::string AppData::jsonDump()
{
    lock_guard lg(m_mtx);

    if (m_updateIdxJsonGen != m_updateIdx)
    {
        m_updateIdxJsonGen = m_updateIdx;

        if (m_procData) { m_procData->updateJsonDump(); }
    }

    json j;

    if (m_procData)
    {
        j = json::parse(m_procData->jsonDump());
        j["appState"] = m_procData->appStateStr();
    }
    else
    {
        j = json(json::value_t::object);
        j["appState"] = appStateStr::boot;
    }

    return j.dump();
}



std::string Status::jsonDump() const
{
    lock_guard lg(m_mtx);



    json devInfo(json::value_t::object);
    devInfo["swVersion"] = prj::version.toString();
    devInfo["hwVersion"] = (otp::data->hardwareVersion().isValid() ? otp::data->hardwareVersion().toString() : "");

    json conn(json::value_t::object);
    conn["backend"] = m_backendConn;
    conn["internet"] = m_inetConn;

    json dbgInfoMax32664Info(json::value_t::object);
    dbgInfoMax32664Info["chipVersion"] = m_max32664Info.chipVersionStr();
    dbgInfoMax32664Info["version"] = m_max32664Info.version().toString();
    dbgInfoMax32664Info["afeType"] = m_max32664Info.afeTypeStr();
    dbgInfoMax32664Info["afeVersion"] = m_max32664Info.afeVersionStr();

    json dbgInfoMax32664(json::value_t::object);
    dbgInfoMax32664["info"] = dbgInfoMax32664Info;

    json dbgInfo(json::value_t::object);
    dbgInfo["threadBootFlags"] = m_thBootFlags;
    dbgInfo["peripheralErrorFlags"] = m_periphErrFlag;
    dbgInfo["max32664"] = dbgInfoMax32664;
    dbgInfo["messages"] = json(json::value_t::array);
    for (size_t i = 0; i < m_dbgmsg.size(); ++i)
    {
        json tmp(json::value_t::object);
        tmp["t"] = m_dbgmsg[i].time();
        tmp["msg"] = m_dbgmsg[i].msg();
        dbgInfo["messages"].push_back(tmp);
    }



    json j(json::value_t::object);
    j["deviceInfo"] = devInfo;
    j["machine_id"] = m_machineId;
    j["debugInfo"] = dbgInfo;
    j["airTemp"] = i2cThread::sd.getTempAir();
    j["connection"] = conn;



    return j.dump();
}

void Status::addDbgMsg(const std::string& msg)
{
    lock_guard lg(m_mtx);

    m_dbgmsg.push_back(DebugMessage(time(nullptr), msg));

    constexpr size_t maxNumOfMsg = 256;

    while (m_dbgmsg.size() > maxNumOfMsg) { m_dbgmsg.erase(m_dbgmsg.begin() + 0); }
}



app::AppData appData = app::AppData();

app::Status status = app::Status();

} // namespace app

``````


### src/application/idle.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/idle.h
`relative_path`: src/application/idle.h
`format`: Arbitrary Binary Data
`size`: 340   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_IDLE_H
#define IG_APP_IDLE_H

#include <cstddef>
#include <cstdint>


namespace app::idle {

enum
{
    running = 0,
    localSetup,
    startTreatment,

    ___ret_end_
};

int task();

}


#endif // IG_APP_IDLE_H

``````


### src/application/appMain.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/appMain.cpp
`relative_path`: src/application/appMain.cpp
`format`: Arbitrary Binary Data
`size`: 25664   




### src/application/dfu.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/dfu.cpp
`relative_path`: src/application/dfu.cpp
`format`: Arbitrary Binary Data
`size`: 15201   




### src/application/localSetup.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/localSetup.cpp
`relative_path`: src/application/localSetup.cpp
`format`: Arbitrary Binary Data
`size`: 25235   




### src/application/treatmentData.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/treatmentData.cpp
`relative_path`: src/application/treatmentData.cpp
`format`: Arbitrary Binary Data
`size`: 763   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>

#include "treatmentData.h"


#define LOG_MODULE_LEVEL LOG_LEVEL_INF
#define LOG_MODULE_NAME  TRETDATA
#include "middleware/log.h"


namespace {}



const app::treat::Block app::treat::endBlock(Block::Type::end, 0, app::treat::defaultTreatmentStartTemp);



namespace app::treat {

const Block& Config::getBlock(blocks_type::size_type idx) const
{
    if (idx < m_blocks.size()) { return m_blocks[idx]; }
    else { return app::treat::endBlock; }
}

uint32_t Config::duration() const
{
    uint32_t dur = 0;

    for (const auto& b : m_blocks) { dur += b.duration(); }

    return dur;
}

} // namespace app::treat

``````


### src/application/commissioning.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/commissioning.h
`relative_path`: src/application/commissioning.h
`format`: Arbitrary Binary Data
`size`: 361   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_COMMISSIONING_H
#define IG_APP_COMMISSIONING_H

#include <cstddef>
#include <cstdint>


namespace app::commissioning {

enum
{
    running = 0,
    done,
    error,

    ___ret_end_
};

int task();

}


#endif // IG_APP_COMMISSIONING_H

``````


### src/application/statusData.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/statusData.cpp
`relative_path`: src/application/statusData.cpp
`format`: Arbitrary Binary Data
`size`: 3604   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>

#include "middleware/json.h"
#include "project.h"
#include "statusData.h"


#define LOG_MODULE_LEVEL LOG_LEVEL_INF
#define LOG_MODULE_NAME  STATUSDATA
#include "middleware/log.h"


namespace {

json toJson(const app::PeltierStatusContainer& pelt)
{
    auto pstoj = [](const app::PeltierStatus& ps) {
        json j(json::value_t::object);
        j["temperature"] = ps.temp();
        j["setPoint"] = ps.setPoint();
        j["tolerance"] = ps.tolerance();
        return j;
    };

    json j(json::value_t::object);

    j["left"] = pstoj(pelt.left);
    j["right"] = pstoj(pelt.right);

    return j;
}

} // namespace



namespace app::appStateStr {

const char* const boot = "booting";
const char* const commission = "commissioning";
const char* const localSetup = "localSetup";
// const char* const prepare = "prepare";
const char* const idle = "idle";
const char* const treatment = "treatment";
const char* const error = "error";

}

namespace app {

ProcessData::ProcessData(const std::string& appStateStr)
    : m_jsonDump(), m_appStateStr(appStateStr)
{
    json j(json::value_t::object);
    m_jsonDump = j.dump();
}

void CommissionData::updateJsonDump()
{
    json j(json::value_t::object);

    j["commissionId"] = m_commissionId;
    j["attempt"] = m_attempt;

    m_jsonDump = j.dump();
}

void LocalSetupData::updateJsonDump()
{
    json system(json::value_t::object);
    system["displayBrightness"] = m_displayBrightness;

    json wifi(json::value_t::object);
    wifi["country"] = m_wifi_country;
    wifi["ssid"] = m_wifi_ssid;

    json j(json::value_t::object);
    j["system"] = system;
    j["wifi"] = wifi;

    m_jsonDump = j.dump();
}

#ifdef ___CLASS_PrepareData_DECLARED
void PrepareData::updateJsonDump()
{
    json j(json::value_t::object);

    j["peltier"] = toJson(m_pelt);

    m_jsonDump = j.dump();
}
#endif // ___CLASS_PrepareData_DECLARED

void IdleData::updateJsonDump()
{
    json j(json::value_t::object);

    m_jsonDump = j.dump();
}

void TreatmentData::set(bool handsOn, int noHandsTmr, uint32_t treatTime, uint32_t elapsedTime, const PeltierStatusContainer& pelt, const biometric::Data& biom)
{
    m_handsOn = handsOn;
    m_noHandsTmr = noHandsTmr;

    m_treatmentTime = treatTime;
    m_elapsedTime = elapsedTime;

    m_pelt = pelt;
    m_biom = biom;
}

void TreatmentData::updateJsonDump()
{
    json j(json::value_t::object);

    j["userName"] = m_userName;
    j["abortReason"] = m_abortReason;
    j["handsOn"] = m_handsOn;
    j["noHandsTimeout"] = m_noHandsTmr;
    j["treatmentDuration"] = m_treatmentDuration;
    j["treatmentTime"] = m_treatmentTime;
    j["elapsedTime"] = m_elapsedTime;

    switch (m_blockType)
    {
    case BlockType::init:
        j["blockType"] = "init";
        break;

    case BlockType::treat:
        j["blockType"] = "treat";
        break;

    case BlockType::pause:
        j["blockType"] = "pause";
        break;

    case BlockType::done:
        j["blockType"] = "done";
        break;
    }

    j["peltier"] = toJson(m_pelt);

    json biom(json::value_t::object);
    biom["heartRate"] = m_biom.heartRate();
    biom["oxygenSaturation"] = m_biom.oxygenSat();
    biom["algorithmState"] = m_biom.algorithmState();
    biom["algorithmStatus"] = m_biom.algorithmStatus();
    j["biometrics"] = biom;

    m_jsonDump = j.dump();
}

void ErrorData::updateJsonDump()
{
    json j(json::value_t::object);

    m_jsonDump = j.dump();
}

} // namespace app

``````


### src/application/idle.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/idle.cpp
`relative_path`: src/application/idle.cpp
`format`: Arbitrary Binary Data
`size`: 3564   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <ctime>

#include "api/api.h"
#include "application/appShared.h"
#include "application/status.h"
#include "application/treatment.h"
#include "idle.h"
#include "middleware/curlThread.h"
#include "middleware/i2cThread.h"
#include "middleware/util.h"
#include "project.h"


#define LOG_MODULE_LEVEL LOG_LEVEL_INF
#define LOG_MODULE_NAME  IDLE
#include "middleware/log.h"


namespace {

enum
{
    S_init = 0,
    S_enter,
    S_exit,
    S_idle,

    S_apiReqStart,
    S_apiAwaitResStart,

    S_apiReqSettings,
    S_apiAwaitResSettings,
};

}



int app::idle::task()
{
    int r = idle::running;

    static int state = S_init;
    static int returnCommand = idle::running;
    static time_t tAttempt = 0;
    static time_t retryDelay = 1;

    const time_t tNow = time(nullptr);

    static int oldState = -1;
    if (oldState != state)
    {
        LOG_DBG("%s state %i -> %i", ___LOG_STR(LOG_MODULE_NAME), oldState, state);

        oldState = state;
    }

    switch (state)
    {
    case S_init:
        state = S_enter;
        break;

    case S_enter:
        app::appData.set(app::IdleData());
        tAttempt = 0;

        state = S_idle;
        break;

    case S_exit:
        LOG_DBG("exit idle task: %i", returnCommand);
        state = S_enter;
        r = returnCommand;
        break;

    case S_idle:
        if ((tNow - tAttempt) >= retryDelay)
        {
#if PRJ_ENTER_LOCALSETUP_FROM_IDLE
            if (app::mouseConnected)
            {
                returnCommand = idle::localSetup;
                state = S_exit;
            }
            else
#endif
            {
                app::appData.set(app::IdleData());

                state = S_apiReqStart;
            }
        }
        break;

    case S_apiReqStart:
        if (api::thread::sd.reqStart(api::StartReq()) == 0) { state = S_apiAwaitResStart; }
        break;

    case S_apiAwaitResStart:
    {
        const auto status = api::thread::sd.status();

        if (status.state() == api::thread::state_res)
        {
            const api::StartRes resData = api::thread::sd.getStartRes();

            if (resData.treatClearance()) { state = S_apiReqSettings; }
            else { state = S_idle; }
        }
        else if (status.state() == api::thread::state_error)
        {
            api::thread::sd.flush();
            state = S_idle;
        }

#if defined(PRJ_DEBUG) && 0
        retryDelay = 1;
#else // PRJ_DEBUG

#if PRJ_START_AT_HANDS_ON
        retryDelay = 0;
#else
        retryDelay = api::thread::getReqInterval(2, 7);
#endif

#endif // PRJ_DEBUG
        tAttempt = tNow;
    }
    break;

    case S_apiReqSettings:
        if (api::thread::sd.reqSettings(api::SettingsReq()) == 0) { state = S_apiAwaitResSettings; }
        break;

    case S_apiAwaitResSettings:
    {
        const auto status = api::thread::sd.status();

        if (status.state() == api::thread::state_res)
        {
            const api::SettingsRes resData = api::thread::sd.getSettingsRes();

            app::treat::setConfig(resData.user(), resData.treatConfig());
            returnCommand = idle::startTreatment;
            state = S_exit;
        }
        else if (status.state() == api::thread::state_error)
        {
            api::thread::sd.flush();
            state = S_idle;
        }
    }
    break;

    default:
        LOG_invalidState(state, 5);
        break;
    }

    return r;
}

``````


### src/application/config.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/config.cpp
`relative_path`: src/application/config.cpp
`format`: Arbitrary Binary Data
`size`: 4984   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

#include "config.h"
#include "middleware/iniFile.h"

#include <omw/string.h>


#define LOG_MODULE_LEVEL LOG_LEVEL_DBG
#define LOG_MODULE_NAME  CONFIG
#include "middleware/log.h"


namespace {

const char* const section_cfg = "Config";
const char* const key_cfg_binVer = "BinVer";
const char* const key_cfg_writeCfgFile = "WriteThisFileToDiskNextTime";

const char* const section_sys = "System";
const char* const key_sys_machineId = "MachineID";
const char* const key_sys_displayBrightness = "DisplayBrightness";
const char* const key_sys_wifiCountry = "WiFiCountry";
const char* const key_sys_timezone = "Timezone";

// const char* const section_ls = "LocalSetup";
// const char* const key_ls_done = "Done";
// const char* const key_ls_state = "State";

const char* const section_api = "API";
const char* const key_api_baseUrl = "BaseUrl";

} // namespace



app::Config::Config()
    : iniFile(), updateResult(0)
{
    iniFile.setLineEnding("\n");
    iniFile.setWriteBom(false);
}

int app::Config::getUpdateResult() const { return updateResult; }

int app::Config::save()
{
    iniFile.setValue(section_cfg, key_cfg_binVer, cfg_binVer.toString());
    iniFile.setValue(section_cfg, key_cfg_writeCfgFile, omw::to_string(cfg_writeCfgFile));

    iniFile.setValue(section_sys, key_sys_machineId, sys_machineId);
    iniFile.setValue(section_sys, key_sys_displayBrightness, omw::to_string(sys_displayBrightness));
    iniFile.setValue(section_sys, key_sys_wifiCountry, sys_wifiCountry);
    iniFile.setValue(section_sys, key_sys_timezone, sys_timezone);

    // iniFile.setValue(section_ls, key_ls_done, omw::to_string(ls_done));
    // iniFile.setValue(section_ls, key_ls_state, ls_state);

    iniFile.setValue(section_api, key_api_baseUrl, api_baseUrl);

    return iniFile.writeFile();
}

int app::Config::read()
{
    int r = iniFile.readFile();

    if (r == 0)
    {
        r = update();
        if (r != 0) r |= 0x80000000;
    }
    else
    {
        r = update();
        if (r != 0) r |= 0x40000000;
        save();
    }

    return r;
}

int app::Config::update()
{
    constexpr int ur_cfg_bit = 0x0001;
    constexpr int ur_sys_bit = 0x0002;
    // constexpr int ur_ls_bit = 0x0008;
    constexpr int ur_api_bit = 0x0004;
    updateResult = 0;

    try
    {
        std::string tmpStr = iniFile.getValueD(section_cfg, key_cfg_binVer, std::string(default_cfg_binVer));
        cfg_binVer = omw::Version(tmpStr);
        if (!cfg_binVer.isValid()) throw(-1);
    }
    catch (...)
    {
        cfg_binVer = omw::Version(std::string(default_cfg_binVer));
        updateResult |= ur_cfg_bit;
    }

    try
    {
        cfg_writeCfgFile = omw::stob(iniFile.getValueD(section_cfg, key_cfg_writeCfgFile, omw::to_string(default_cfg_writeCfgFile)));
    }
    catch (...)
    {
        // cfg_writeCfgFile = default_cfg_writeCfgFile;
        // updateResult |= ur_cfg_bit;
        //
        // special case

        cfg_writeCfgFile = true;
    }



    try
    {
        sys_machineId = iniFile.getValueD(section_sys, key_sys_machineId, std::string(default_sys_machineId));
    }
    catch (...)
    {
        sys_machineId = default_sys_machineId;
        updateResult |= ur_sys_bit;
    }

    try
    {
        sys_displayBrightness = std::stoi((iniFile.getValueD(section_sys, key_sys_displayBrightness, omw::to_string(default_sys_displayBrightness))));
    }
    catch (...)
    {
        sys_displayBrightness = default_sys_displayBrightness;
        updateResult |= ur_sys_bit;
    }

    try
    {
        sys_wifiCountry = iniFile.getValueD(section_sys, key_sys_wifiCountry, std::string(default_sys_wifiCountry));
    }
    catch (...)
    {
        sys_wifiCountry = default_sys_wifiCountry;
        updateResult |= ur_sys_bit;
    }

    try
    {
        sys_timezone = iniFile.getValueD(section_sys, key_sys_timezone, std::string(default_sys_timezone));
    }
    catch (...)
    {
        sys_timezone = default_sys_timezone;
        updateResult |= ur_sys_bit;
    }



    // try
    //{
    //     ls_done = omw::stob(iniFile.getValueD(section_ls, key_ls_done, omw::to_string(default_ls_done)));
    // }
    // catch (...)
    //{
    //     ls_done = default_ls_done;
    //     updateResult |= ur_ls_bit;
    // }
    //
    // try
    //{
    //    ls_state = iniFile.getValueD(section_ls, key_ls_state, default_ls_state.data());
    //}
    // catch (...)
    //{
    //    ls_state = default_ls_state;
    //    updateResult |= ur_ls_bit;
    //}



    try
    {
        api_baseUrl = iniFile.getValueD(section_api, key_api_baseUrl, std::string(default_api_baseUrl));
    }
    catch (...)
    {
        api_baseUrl = default_api_baseUrl.data();
        updateResult |= ur_api_bit;
    }



    return updateResult;
}



app::Config app::config = app::Config();

``````


### src/application/appMain.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/appMain.h
`relative_path`: src/application/appMain.h
`format`: Arbitrary Binary Data
`size`: 773   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_APPMAIN_H
#define IG_APP_APPMAIN_H

#include <cstddef>
#include <cstdint>


enum EXITCODE // https://tldp.org/LDP/abs/html/exitcodes.html / on MSW are no preserved codes
{
    EC_OK = 0,
    EC_ERROR = 1,

    EC__begin_ = 79,

    EC_RPIHAL_INIT_ERROR = EC__begin_,
    // EC_..,

    EC__end_,

    EC__max_ = 113
};
static_assert(EC__end_ <= EC__max_, "too many error codes defined");
#if (defined(__unix__) || defined(__unix))
#include "sysexits.h"
static_assert(EC__begin_ == EX__MAX + 1, "maybe the first allowed user code has to be changed");
#endif // UNIX


namespace app {

int appMain();

} // namespace app


#endif // IG_APP_APPMAIN_H

``````


### src/application/appShared.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/appShared.h
`relative_path`: src/application/appShared.h
`format`: Arbitrary Binary Data
`size`: 460   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

/*

This is not shared between threads, it's only used in the main thread. So mutex and atomic is not needed.

*/

#ifndef IG_APP_APPSHARED_H
#define IG_APP_APPSHARED_H

#include <cstddef>
#include <cstdint>



namespace app {

/**
 * @brief Debounced mouse connection status.
 */
extern const bool& mouseConnected;

}


#endif // IG_APP_APPSHARED_H

``````


### src/application/localSetup.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/localSetup.h
`relative_path`: src/application/localSetup.h
`format`: Arbitrary Binary Data
`size`: 349   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_LOCALSETUP_H
#define IG_APP_LOCALSETUP_H

#include <cstddef>
#include <cstdint>


namespace app::localSetup {

enum
{
    running = 0,
    done,
    error,

    ___ret_end_
};

int task();

}


#endif // IG_APP_LOCALSETUP_H

``````


### src/application/dfu.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/dfu.h
`relative_path`: src/application/dfu.h
`format`: Arbitrary Binary Data
`size`: 331   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_DFU_H
#define IG_APP_DFU_H

#include <cstddef>
#include <cstdint>


namespace app::dfu {

enum
{
    running = 0,
    nop,
    done,
    failed,

    ___ret_end_
};

int task();

}


#endif // IG_APP_DFU_H

``````


### src/application/statusData.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/statusData.h
`relative_path`: src/application/statusData.h
`format`: Arbitrary Binary Data
`size`: 7668   




### src/application/treatment.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/application/treatment.h
`relative_path`: src/application/treatment.h
`format`: Arbitrary Binary Data
`size`: 532   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#ifndef IG_APP_TREATMENT_H
#define IG_APP_TREATMENT_H

#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

#include "application/treatmentData.h"



namespace app::treat {

void setPeltierIdle();
void setConfig(const app::treat::User& user, const app::treat::Config& cfg);

enum
{
    running = 0,
    done,

    ___ret_end_
};

int task();

} // namespace app::treat


#endif // IG_APP_TREATMENT_H

``````


### src/main.cpp
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/machine-firmware/src/main.cpp
`relative_path`: src/main.cpp
`format`: Arbitrary Binary Data
`size`: 2632   


``````
/*
author          Oliver Blaser
copyright       Copyright (c) 2024 Hengartner Elektronik AG and SiLAB AG
*/

#include <cstddef>
#include <cstdint>
#include <iostream>
#include <string>
#include <vector>

#include "application/appMain.h"
#include "middleware/gpio.h"
#include "project.h"

#include <omw/cli.h>
#include <rpihal/rpihal.h>
#include <unistd.h>

#define LOG_MODULE_LEVEL LOG_LEVEL_DBG
#define LOG_MODULE_NAME  MAIN
#include "middleware/log.h"


namespace {}



int main(int argc, char** argv)
{
    int r = EC_OK;

#ifndef PRJ_DEBUG
    if (prj::version.isPreRelease()) { std::cout << omw::fgBrightMagenta << "pre-release v" << prj::version.toString() << omw::defaultForeColor << std::endl; }
#endif

    RPIHAL___setModel___(RPIHAL_model_4B); // temporary hack!       TODO update rpihal
#ifdef RPIHAL_EMU
    if (RPIHAL_EMU_init(RPIHAL_model_4B) == 0)
    {
        while (!RPIHAL_EMU_isRunning()) {}
    }
#endif // RPIHAL_EMU

    if (GPIO_init() == 0)
    {
#if defined(PRJ_DEBUG) && 1
        gpio::dispBacklightEn->write(0);
#endif

        // disable peltier power and pump
        gpio::peltPowerL->write(0);
        gpio::peltPowerR->write(0);
        gpio::pumpA->write(0);
        gpio::pumpB->write(0);
    }
    else { r = EC_RPIHAL_INIT_ERROR; }

#if defined(PRJ_DEBUG) && 0
    RPIHAL_GPIO_dumpAltFuncReg(0x3c0000);
    RPIHAL_GPIO_dumpPullUpDnReg(0x3c0000);
#endif

#if (LOG_MODULE_LEVEL >= LOG_LEVEL_INF)
    {
        const char* dbgStr = "";
#ifdef PRJ_DEBUG
        dbgStr = LOG_SGR_BMAGENTA " DEBUG";
#endif
        LOG_INF(LOG_SGR_BYELLOW "%s " LOG_SGR_BCYAN "v%s%s", prj::appName, prj::version.toString().c_str(), dbgStr);
    }
#endif

    LOG_DBG("pid: %i", (int)getpid());

    while (r == EC_OK)
    {
        gpio::task();

        r = app::appMain();

        usleep(5 * 1000);

#ifdef RPIHAL_EMU
        if (!RPIHAL_EMU_isRunning())
        {
            LOG_WRN("rpihal emu terminate");
            r = EC_OK;
            break;
        }
#endif // RPIHAL_EMU
    }

    gpio::led::run->clr();

    GPIO_failsafe();

#ifdef RPIHAL_EMU
    RPIHAL_EMU_cleanup();

    // wait to prevent segmentation fault
    // util::sleep(10000); doesn't help
#endif // RPIHAL_EMU

    return r;
}



#if !defined(PRJ_DEBUG)

#if PRJ_USE_HTML_IO_EMU || PRJ_PLAIN_RASPI
#error "PRJ_USE_HTML_IO_EMU and PRJ_PLAIN_RASPI have to be disabled in release build"
#endif

#endif // release

#if PRJ_PLAIN_RASPI && GPIO_EN_RESIN0
#warning "GPIO_EN_RESIN0 is enabled while using PRJ_PLAIN_RASPI"
#endif

#if PRJ_PKG_DEV_DISPDMY && !PRJ_USE_HTML_IO_EMU
#warning "vacuul-dev-dispdmy usually has IO emu enabled"
#endif

``````



## machine-frontend
`clone_url`: https://github.com/vacuul-dev/machine-frontend.git



## backend-device-interface-v1
`clone_url`: https://github.com/vacuul-dev/backend-device-interface-v1.git


### machine_error.go
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/backend-device-interface-v1/machine_error.go
`relative_path`: machine_error.go
`format`: Arbitrary Binary Data
`size`: 757   


``````
package handler

import (
	"github.com/appwrite/sdk-for-go/databases"
	"github.com/open-runtimes/types-for-go/v4/openruntimes"
)

type ApiMachinesErrorInput struct {
	MachineID string       `json:"machine_id"`
	Status    StatusStruct `json:"status"`
}

func ApiMachinesError(Context openruntimes.Context, databases *databases.Databases) openruntimes.Response {
	var input ApiMachinesErrorInput

	if err := Context.Req.BodyJson(&input); err != nil {
		Context.Log("Response error: " + "Invalid request body")
		return Context.Res.Json(Json{
			"error": "Invalid request body",
		})
	}

	machineID := input.MachineID
	status := input.Status

	Context.Log("Machine ID: ", machineID)
	Context.Log("Machine Status: ", status)

	return Context.Res.Json(Json{})
}

``````


### go.mod
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/backend-device-interface-v1/go.mod
`relative_path`: go.mod
`format`: Arbitrary Binary Data
`size`: 143   


``````
module openruntimes/handler

go 1.23.0

require github.com/open-runtimes/types-for-go/v4 v4.0.7

require github.com/appwrite/sdk-for-go v0.3.0

``````


### machine_commissioning.go
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/backend-device-interface-v1/machine_commissioning.go
`relative_path`: machine_commissioning.go
`format`: Arbitrary Binary Data
`size`: 1995   


``````
package handler

import (
	"os"

	"github.com/appwrite/sdk-for-go/databases"
	"github.com/appwrite/sdk-for-go/query"
	"github.com/open-runtimes/types-for-go/v4/openruntimes"
)

type ApiMachinesCommissioningInput struct {
	CommissionID string       `json:"commissionId"`
	Status       StatusStruct `json:"status"`
}

func ApiMachinesCommissioning(Context openruntimes.Context, databases *databases.Databases) openruntimes.Response {
	var input ApiMachinesCommissioningInput

	if err := Context.Req.BodyJson(&input); err != nil {
		Context.Log("Response error: " + "Invalid request body")
		return Context.Res.Json(Json{
			"error": "Invalid request body",
		})
	}

	commissionID := input.CommissionID
	status := input.Status

	Context.Log("Commission ID: ", commissionID)
	Context.Log("Machine Status: ", status)

	doc, err := databases.ListDocuments(
		os.Getenv("APPWRITE_DATABASE_ID"),
		os.Getenv("APPWRITE_MACHINES_COLLECTION_ID"),
		databases.WithListDocumentsQueries([]string{
			query.Equal("assigned_commission_id", commissionID),
		}),
	)
	if err != nil {
		Context.Log("Response message: " + "Machine not found")
		Context.Log("Response error: " + err.Error())
		return Context.Res.Json(Json{
			// "message": "Machine not found",
			"status":      0,
			"machine_id":  "",
			"timezone":    "Europe/Zurich",
			"wifiCountry": "CH",
		})
	}

	if len(doc.Documents) == 0 {
		Context.Log("Response message: " + "Machine not found")
		return Context.Res.Json(Json{
			// "message": "Machine not found",
			"status":      0,
			"machine_id":  "",
			"timezone":    "Europe/Zurich",
			"wifiCountry": "CH",
		})
	}

	Context.Log("doc.Documents[0]: ", doc.Documents[0])

	firstMachine := doc.Documents[0]

	machineID := firstMachine.Id
	Context.Log("Machine ID: ", machineID)

	Context.Log("Response message: " + "Machine commissioning successful")
	return Context.Res.Json(Json{
		"status":      1,
		"machine_id":  machineID,
		"timezone":    "Europe/Zurich",
		"wifiCountry": "CH",
	})
}

``````


### machine_start.go
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/backend-device-interface-v1/machine_start.go
`relative_path`: machine_start.go
`format`: Arbitrary Binary Data
`size`: 4087   


``````
package handler

import (
	"os"

	"github.com/appwrite/sdk-for-go/databases"
	"github.com/open-runtimes/types-for-go/v4/openruntimes"
)

type ApiMachinesStartInput struct {
	MachineID string       `json:"machine_id"`
	Status    StatusStruct `json:"status"`
}

func ApiMachinesStart(Context openruntimes.Context, databases *databases.Databases) openruntimes.Response {
	var input ApiMachinesStartInput

	if err := Context.Req.BodyJson(&input); err != nil {
		Context.Log("Response error: " + "Invalid request body")
		return Context.Res.Json(Json{
			"error": "Invalid request body",
		})
	}

	machineID := input.MachineID
	status := input.Status

	Context.Log("Machine ID: ", machineID)
	Context.Log("Machine Status: ", status)

	doc, err := databases.GetDocument(
		os.Getenv("APPWRITE_DATABASE_ID"),
		os.Getenv("APPWRITE_MACHINES_COLLECTION_ID"),
		machineID,
	)
	if err != nil {
		Context.Log("Response message: " + "Machine not found")
		Context.Log("Response error: " + err.Error())
		return Context.Res.Json(Json{
			// "message": "Machine not found",
			"status": 0,
		})
	}

	machineDoc := MachineDocument{}
	if err := doc.Decode(&machineDoc); err != nil {
		Context.Log("Response message: " + "Machine document decoding error")
		return Context.Res.Json(Json{
			// "message": "Machine document decoding error",
			"status": 0,
		})
	}

	if machineDoc.AssignedAppointment == nil || *machineDoc.AssignedAppointment == "" {
		Context.Log("Response message: " + "Machine not assigned to any appointment")
		return Context.Res.Json(Json{
			// "message": "Machine not assigned to any appointment",
			"status": 0,
		})
	}

	if machineDoc.Status == "active" {
		Context.Log("Response message: " + "Treatment already started")
		return Context.Res.Json(Json{
			// "message": "Treatment already started",
			"status": 0,
		})
	}

	appointment, err := databases.GetDocument(
		os.Getenv("APPWRITE_DATABASE_ID"),
		os.Getenv("APPWRITE_APPOINTMENTS_COLLECTION_ID"),
		*machineDoc.AssignedAppointment,
	)
	if err != nil {
		Context.Log("Response message: " + "Appointment not found")
		return Context.Res.Json(Json{
			// "message": "Appointment not found",
			"status": 0,
		})
	}

	Context.Log("Appointment (doc): ", appointment)

	appointmentDoc := AppointmentDocument{}
	if err := appointment.Decode(&appointmentDoc); err != nil {
		Context.Log("Response message: " + "Appointment document decoding error")
		Context.Log("Response error: " + err.Error())
		return Context.Res.Json(Json{
			// "message": "Appointment document decoding error",
			"status": 0,
		})
	}

	if appointmentDoc.State != "acquired" {
		Context.Log("Response message: " + "Appointment not in acquired state")
		// return Context.Res.Json(Json{
		// 	// "message": "Appointment not in acquired state",
		// 	"status": 0,
		// })

		if appointmentDoc.State == "aborted" {
			Context.Log("Response message: " + "Appointment aborted")

			return Context.Res.Json(Json{
				// "message": "Appointment aborted",
				"status": 0,
			})

		}
	}

	// Update machine status
	_, err = databases.UpdateDocument(
		os.Getenv("APPWRITE_DATABASE_ID"),
		os.Getenv("APPWRITE_MACHINES_COLLECTION_ID"),
		machineID,
		databases.WithUpdateDocumentData(map[string]any{
			"status": "active",
		}),
	)
	if err != nil {
		Context.Log("Response message: " + "Machine status update error")
		return Context.Res.Json(Json{
			// "message": "Machine status update error",
			"status": 0,
		})
	}

	// Update appointment state
	_, err = databases.UpdateDocument(
		os.Getenv("APPWRITE_DATABASE_ID"),
		os.Getenv("APPWRITE_APPOINTMENTS_COLLECTION_ID"),
		*machineDoc.AssignedAppointment,
		databases.WithUpdateDocumentData(map[string]any{
			"state": "running",
		}),
	)
	if err != nil {
		Context.Log("Response message: " + "Appointment state update error")
		return Context.Res.Json(Json{
			// "message": "Appointment state update error",
			"status": 0,
		})
	}

	Context.Log("Response message: " + "Treatment authorized to start")
	return Context.Res.Json(Json{
		// "message": "Treatment authorized to start",
		"status": 1,
	})
}

``````


### go.sum
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/backend-device-interface-v1/go.sum
`relative_path`: go.sum
`format`: Arbitrary Binary Data
`size`: 965   


``````
github.com/appwrite/sdk-for-go v0.0.1-rc.2 h1:kh8p6OmSgA4d7aT1KXE9Z3W99ioDKdhhY1OrKsTLu1I=
github.com/appwrite/sdk-for-go v0.0.1-rc.2/go.mod h1:aFiOAbfOzGS3811eMCt3T9WDBvjvPVAfOjw10Vghi4E=
github.com/appwrite/sdk-for-go v0.3.0 h1:5Fun3dRSvsIChUBr2cMJUVRuHZ0skqUQVRQ5u+IWfW8=
github.com/appwrite/sdk-for-go v0.3.0/go.mod h1:aFiOAbfOzGS3811eMCt3T9WDBvjvPVAfOjw10Vghi4E=
github.com/open-runtimes/types-for-go/v4 v4.0.5 h1:Lw82fnOoaEmGgOoQmJhEOsYjs6Ginie4TdS2RYsvayk=
github.com/open-runtimes/types-for-go/v4 v4.0.5/go.mod h1:ab4mDSfgeG4kN8wWpaBSv0Ao3m9P6oEfN5gsXtx+iaI=
github.com/open-runtimes/types-for-go/v4 v4.0.6 h1:0Xf58LMy/vwWkiRN6BvvpWt1mWzcWUWQ5wsWSezG2TU=
github.com/open-runtimes/types-for-go/v4 v4.0.6/go.mod h1:ab4mDSfgeG4kN8wWpaBSv0Ao3m9P6oEfN5gsXtx+iaI=
github.com/open-runtimes/types-for-go/v4 v4.0.7 h1:yIkdY8Q1438mVBgCO77pKLXfD2UOsMga5H0VVeMXEO8=
github.com/open-runtimes/types-for-go/v4 v4.0.7/go.mod h1:L6N4mNQFedFSlX5hOKmtjUTZdqD691YZlVtXOxC5DSw=

``````


### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/backend-device-interface-v1/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 869   


``````
# ⚡ Go Starter Function

A simple starter function. Edit `src/main.go` to get started and create something awesome! 🚀

## 🧰 Usage

### GET /ping

- Returns a "Pong" message.

**Response**

Sample `200` Response:

```text
Pong
```

### GET, POST, PUT, PATCH, DELETE /

- Returns a "Learn More" JSON response.

**Response**

Sample `200` Response:

```json
{
  "motto": "Build like a team of hundreds_",
  "learn": "https://appwrite.io/docs",
  "connect": "https://appwrite.io/discord",
  "getInspired": "https://builtwith.appwrite.io"
}
```

## ⚙️ Configuration

| Setting           | Value         |
| ----------------- | ------------- |
| Runtime           | Go (1.23)     |
| Entrypoint        | `main.go`     |
| Permissions       | `any`         |
| Timeout (Seconds) | 15            |

## 🔒 Environment Variables

No environment variables required.

``````


### machine_progress.go
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/backend-device-interface-v1/machine_progress.go
`relative_path`: machine_progress.go
`format`: Arbitrary Binary Data
`size`: 5209   




### structs.go
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/backend-device-interface-v1/structs.go
`relative_path`: structs.go
`format`: Arbitrary Binary Data
`size`: 2677   


``````
package handler

type MachineDocument struct {
	ID                   string  `json:"$id"`
	Status               string  `json:"status"`
	AssignedAppointment  *string `json:"assigned_appointment"`
	AssignedCommissionID *string `json:"assigned_commission_id"`
}

type AppointmentDocument struct {
	State     string `json:"state"`
	MachineId string `json:"machineId"`
	// TimeSlotId *string `json:"timeSlotId"`
	// PaymentId  *string `json:"paymentId"`
	TherapyId *string `json:"therapyId"`
}

type TherapyDocument struct {
	AppointmentId string `json:"appointmentId"`
	Settings      string `json:"settings"`
	Active        bool   `json:"active"`
}

type Json = map[string]any

type ProgressAppData struct {
	AppData struct {
		AbortReason int    `json:"abortReason"`
		AppState    string `json:"appState"`
		Biometrics  struct {
			AlgorithmState   int     `json:"algorithmState"`
			AlgorithmStatus  int     `json:"algorithmStatus"`
			HeartRate        float64 `json:"heartRate"`
			OxygenSaturation float64 `json:"oxygenSaturation"`
		} `json:"biometrics"`
		BlockType      string `json:"blockType"`
		ElapsedTime    int    `json:"elapsedTime"`
		HandsOn        bool   `json:"handsOn"`
		NoHandsTimeout int    `json:"noHandsTimeout"`
		Peltier        struct {
			Left struct {
				SetPoint    float64 `json:"setPoint"`
				Temperature float64 `json:"temperature"`
				Tolerance   float64 `json:"tolerance"`
			} `json:"left"`
			Right struct {
				SetPoint    float64 `json:"setPoint"`
				Temperature float64 `json:"temperature"`
				Tolerance   float64 `json:"tolerance"`
			} `json:"right"`
		} `json:"peltier"`
		TreatmentDuration int    `json:"treatmentDuration"`
		TreatmentTime     int    `json:"treatmentTime"`
		UserName          string `json:"userName"`
	} `json:"appData"`
	MachineID string       `json:"machine_id"`
	Status    StatusStruct `json:"status"`
}

type StatusStruct struct {
	AirTemp    float64 `json:"airTemp"`
	Connection struct {
		Backend  bool `json:"backend"`
		Internet bool `json:"internet"`
	} `json:"connection"`
	DebugInfo struct {
		Max32664 struct {
			Info struct {
				AfeType     string `json:"afeType"`
				AfeVersion  string `json:"afeVersion"`
				ChipVersion string `json:"chipVersion"`
				Version     string `json:"version"`
			} `json:"info"`
		} `json:"max32664"`
		Messages             []interface{} `json:"messages"`
		PeripheralErrorFlags int           `json:"peripheralErrorFlags"`
		ThreadBootFlags      int           `json:"threadBootFlags"`
	} `json:"debugInfo"`
	DeviceInfo struct {
		HwVersion string `json:"hwVersion"`
		SwVersion string `json:"swVersion"`
	} `json:"deviceInfo"`
	MachineID string `json:"machine_id"`
}

``````


### machine_settings.go
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/backend-device-interface-v1/machine_settings.go
`relative_path`: machine_settings.go
`format`: Arbitrary Binary Data
`size`: 4237   


``````
package handler

import (
	"os"
	"regexp"
	"strconv"

	"github.com/appwrite/sdk-for-go/databases"
	"github.com/open-runtimes/types-for-go/v4/openruntimes"
)

type ApiMachinesSettingsInput struct {
	MachineID string       `json:"machine_id"`
	Status    StatusStruct `json:"status"`
}

func ApiMachinesSettings(Context openruntimes.Context, databases *databases.Databases) openruntimes.Response {
	var input ApiMachinesSettingsInput

	if err := Context.Req.BodyJson(&input); err != nil {
		Context.Log("Response error: " + "Invalid request body")
		return Context.Res.Json(Json{
			"error": "Invalid request body",
		})
	}

	machineID := input.MachineID
	status := input.Status

	Context.Log("Machine ID: ", machineID)
	Context.Log("Machine Status: ", status)

	doc, err := databases.GetDocument(
		os.Getenv("APPWRITE_DATABASE_ID"),
		os.Getenv("APPWRITE_MACHINES_COLLECTION_ID"),
		machineID,
	)
	if err != nil {
		Context.Log("Response message: " + "Machine not found")
		return Context.Res.Json(Json{
			"message": "Machine not found",
		})
	}

	machineDoc := MachineDocument{}
	if err := doc.Decode(&machineDoc); err != nil {
		Context.Log("Response message: " + "Machine document decoding error")
		return Context.Res.Json(Json{
			"message": "Machine document decoding error",
		})
	}

	if machineDoc.AssignedAppointment == nil || *machineDoc.AssignedAppointment == "" {
		Context.Log("Response message: " + "Machine not assigned to any appointment")
		return Context.Res.Json(Json{
			"message": "Machine not assigned to any appointment",
		})
	}

	appointment, err := databases.GetDocument(
		os.Getenv("APPWRITE_DATABASE_ID"),
		os.Getenv("APPWRITE_APPOINTMENTS_COLLECTION_ID"),
		*machineDoc.AssignedAppointment,
	)
	if err != nil {
		Context.Log("Response message: " + "Appointment not found")
		return Context.Res.Json(Json{
			"message": "Appointment not found",
		})
	}

	Context.Log("Appointment (doc): ", appointment)

	appointmentDoc := AppointmentDocument{}
	if err := appointment.Decode(&appointmentDoc); err != nil {
		Context.Log("Response message: " + "Appointment document decoding error")
		Context.Log("Response error: " + err.Error())
		return Context.Res.Json(Json{
			"message": "Appointment document decoding error",
		})
	}

	Context.Log("Appointment: ", appointmentDoc)

	if appointmentDoc.TherapyId == nil {
		Context.Log("Response message: " + "Therapy not assigned to the appointment")
		return Context.Res.Json(Json{
			"message": "Therapy not assigned to the appointment",
		})
	}

	therapyId := *appointmentDoc.TherapyId

	Context.Log("Therapy ID: ", therapyId)

	therapy, err := databases.GetDocument(
		os.Getenv("APPWRITE_DATABASE_ID"),
		os.Getenv("APPWRITE_THERAPIES_COLLECTION_ID"),
		therapyId,
	)
	if err != nil {
		Context.Log("Response message: " + "Therapy not found")
		return Context.Res.Json(Json{
			"message": "Therapy not found",
		})
	}

	Context.Log("Therapy: ", therapy)

	therapyDoc := TherapyDocument{}
	if err := therapy.Decode(&therapyDoc); err != nil {
		Context.Log("Response message: " + "Therapy document decoding error")
		Context.Log("Response error: " + err.Error())
		return Context.Res.Json(Json{
			"message": "Therapy document decoding error",
		})
	}

	Context.Log("Therapy Settings: ", therapyDoc.Settings)

	re := regexp.MustCompile(`"lightColor":"([^"]+)"\s*,\s*"temperature":(\d+)\s*,\s*"rounds":(\d+)`)
	matches := re.FindStringSubmatch(therapyDoc.Settings)
	if len(matches) != 4 {
		Context.Log("Response message: " + "Machine settings decoding error")
	}

	lightColor := matches[1]
	if lightColor == "" {
		lightColor = "#1E90FF"
	}

	temperature, err := strconv.Atoi(matches[2])
	if err != nil {
		temperature = 10
	}

	rounds, err := strconv.Atoi(matches[3])
	if err != nil {
		rounds = 4
	}

	totalDuration := 480

	calculatedBlocks := []Json{}
	for i := 0; i < rounds; i++ {
		block := Json{
			"type":        i % 2,
			"temperature": float64(temperature),
			"duration":    totalDuration / rounds,
		}

		calculatedBlocks = append(calculatedBlocks, block)
	}

	return Context.Res.Json(Json{
		"user": Json{
			"name": "John Doe",
		},
		"treatment": Json{
			"led": Json{
				"static-colour": lightColor,
			},
			"blocks": calculatedBlocks,
		},
	})
}

``````


### main.go
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/backend-device-interface-v1/main.go
`relative_path`: main.go
`format`: Arbitrary Binary Data
`size`: 1144   


``````
package handler

import (
	"os"

	"github.com/appwrite/sdk-for-go/appwrite"
	"github.com/open-runtimes/types-for-go/v4/openruntimes"
)

func Main(Context openruntimes.Context) openruntimes.Response {
	client := appwrite.NewClient(
		appwrite.WithEndpoint(os.Getenv("APPWRITE_FUNCTION_API_ENDPOINT")),
		appwrite.WithProject(os.Getenv("APPWRITE_FUNCTION_PROJECT_ID")),
		appwrite.WithJWT(Context.Req.Headers["x-appwrite-user-jwt"]),
	)

	databases := appwrite.NewDatabases(client)

	Context.Log("Request Path: " + Context.Req.Path)
	Context.Log("Request Body: " + Context.Req.BodyRaw())

	switch Context.Req.Path {
	case "/api/machines":
		return ApiMachinesCommissioning(Context, databases)
	case "/api/machines/start":
		return ApiMachinesStart(Context, databases)
	case "/api/machines/settings":
		return ApiMachinesSettings(Context, databases)
	case "/api/machines/progress":
		return ApiMachinesProgress(Context, databases)
	case "/api/machines/error":
		return ApiMachinesError(Context, databases)
	}

	return Context.Res.Json(Json{
		"path":    Context.Req.Path,
		"message": "Route not mapped",
		"payload": Context.Req.BodyRaw(),
	})
}

``````



## app
`clone_url`: https://github.com/vacuul-dev/app.git


### test/widget_test.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/test/widget_test.dart
`relative_path`: test/widget_test.dart
`format`: Arbitrary Binary Data
`size`: 1066   


``````
// This is a basic Flutter widget test.
//
// To perform an interaction with a widget in your test, use the WidgetTester
// utility in the flutter_test package. For example, you can send tap and scroll
// gestures. You can also use WidgetTester to find child widgets in the widget
// tree, read text, and verify that the values of widget properties are correct.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:vacuul_reworked/main.dart';

void main() {
  testWidgets('Counter increments smoke test', (WidgetTester tester) async {
    // Build our app and trigger a frame.
    await tester.pumpWidget(const MyApp());

    // Verify that our counter starts at 0.
    expect(find.text('0'), findsOneWidget);
    expect(find.text('1'), findsNothing);

    // Tap the '+' icon and trigger a frame.
    await tester.tap(find.byIcon(Icons.add));
    await tester.pump();

    // Verify that our counter has incremented.
    expect(find.text('0'), findsNothing);
    expect(find.text('1'), findsOneWidget);
  });
}

``````


### pubspec.lock
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/pubspec.lock
`relative_path`: pubspec.lock
`format`: Arbitrary Binary Data
`size`: 30898   




### ios/Runner.xcworkspace/contents.xcworkspacedata
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner.xcworkspace/contents.xcworkspacedata
`relative_path`: ios/Runner.xcworkspace/contents.xcworkspacedata
`format`: Extensible Markup Language
`size`: 224   




### ios/Runner.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`relative_path`: ios/Runner.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`format`: Extensible Markup Language
`size`: 238   




### ios/Runner.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`relative_path`: ios/Runner.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`format`: Extensible Markup Language
`size`: 226   




### ios/RunnerTests/RunnerTests.swift
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/RunnerTests/RunnerTests.swift
`relative_path`: ios/RunnerTests/RunnerTests.swift
`format`: Arbitrary Binary Data
`size`: 285   


``````
import Flutter
import UIKit
import XCTest

class RunnerTests: XCTestCase {

  func testExample() {
    // If you add code to the Runner application, consider adding tests here.
    // See https://developer.apple.com/documentation/xctest for more information about using XCTest.
  }

}

``````


### ios/Runner/Runner-Bridging-Header.h
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Runner-Bridging-Header.h
`relative_path`: ios/Runner/Runner-Bridging-Header.h
`format`: Arbitrary Binary Data
`size`: 38   


``````
#import "GeneratedPluginRegistrant.h"

``````


### ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@2x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@2x.png
`relative_path`: ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@2x.png
`format`: Portable Network Graphics
`size`: 68   




### ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@3x.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@3x.png
`relative_path`: ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage@3x.png
`format`: Portable Network Graphics
`size`: 68   




### ios/Runner/Assets.xcassets/LaunchImage.imageset/README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/LaunchImage.imageset/README.md
`relative_path`: ios/Runner/Assets.xcassets/LaunchImage.imageset/README.md
`format`: Arbitrary Binary Data
`size`: 336   


``````
# Launch Screen Assets

You can customize the launch screen with your own desired assets by replacing the image files in this directory.

You can also do it by opening your Flutter project's Xcode project with `open ios/Runner.xcworkspace`, selecting `Runner/Assets.xcassets` in the Project Navigator and dropping in the desired images.
``````


### ios/Runner/Assets.xcassets/LaunchImage.imageset/Contents.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/LaunchImage.imageset/Contents.json
`relative_path`: ios/Runner/Assets.xcassets/LaunchImage.imageset/Contents.json
`format`: Arbitrary Binary Data
`size`: 391   


``````
{
  "images" : [
    {
      "idiom" : "universal",
      "filename" : "LaunchImage.png",
      "scale" : "1x"
    },
    {
      "idiom" : "universal",
      "filename" : "LaunchImage@2x.png",
      "scale" : "2x"
    },
    {
      "idiom" : "universal",
      "filename" : "LaunchImage@3x.png",
      "scale" : "3x"
    }
  ],
  "info" : {
    "version" : 1,
    "author" : "xcode"
  }
}

``````


### ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage.png
`relative_path`: ios/Runner/Assets.xcassets/LaunchImage.imageset/LaunchImage.png
`format`: Portable Network Graphics
`size`: 68   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/88.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/88.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/88.png
`format`: Joint Photographic Experts Group
`size`: 4540   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/1024.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/1024.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/1024.png
`format`: Joint Photographic Experts Group
`size`: 242811   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/76.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/76.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/76.png
`format`: Joint Photographic Experts Group
`size`: 3984   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/60.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/60.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/60.png
`format`: Joint Photographic Experts Group
`size`: 3187   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/48.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/48.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/48.png
`format`: Joint Photographic Experts Group
`size`: 2412   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/216.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/216.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/216.png
`format`: Joint Photographic Experts Group
`size`: 14328   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/64.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/64.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/64.png
`format`: Joint Photographic Experts Group
`size`: 3188   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/58.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/58.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/58.png
`format`: Joint Photographic Experts Group
`size`: 2980   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/167.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/167.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/167.png
`format`: Joint Photographic Experts Group
`size`: 10089   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/72.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/72.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/72.png
`format`: Joint Photographic Experts Group
`size`: 3681   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/66.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/66.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/66.png
`format`: Joint Photographic Experts Group
`size`: 3382   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/172.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/172.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/172.png
`format`: Joint Photographic Experts Group
`size`: 10650   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/29.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/29.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/29.png
`format`: Joint Photographic Experts Group
`size`: 1706   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/100.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/100.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/100.png
`format`: Joint Photographic Experts Group
`size`: 5275   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/114.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/114.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/114.png
`format`: Joint Photographic Experts Group
`size`: 5982   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/128.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/128.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/128.png
`format`: Joint Photographic Experts Group
`size`: 6739   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/102.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/102.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/102.png
`format`: Joint Photographic Experts Group
`size`: 5265   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/512.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/512.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/512.png
`format`: Joint Photographic Experts Group
`size`: 52339   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/16.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/16.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/16.png
`format`: Joint Photographic Experts Group
`size`: 1072   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/258.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/258.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/258.png
`format`: Joint Photographic Experts Group
`size`: 18784   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/108.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/108.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/108.png
`format`: Joint Photographic Experts Group
`size`: 5779   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/120.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/120.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/120.png
`format`: Joint Photographic Experts Group
`size`: 6265   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/256.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/256.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/256.png
`format`: Joint Photographic Experts Group
`size`: 18448   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/20.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/20.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/20.png
`format`: Joint Photographic Experts Group
`size`: 1325   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/32.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/32.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/32.png
`format`: Joint Photographic Experts Group
`size`: 1709   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/Contents.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/Contents.json
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/Contents.json
`format`: Arbitrary Binary Data
`size`: 7817   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/180.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/180.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/180.png
`format`: Joint Photographic Experts Group
`size`: 11205   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/234.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/234.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/234.png
`format`: Joint Photographic Experts Group
`size`: 16466   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/57.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/57.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/57.png
`format`: Joint Photographic Experts Group
`size`: 2881   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/80.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/80.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/80.png
`format`: Joint Photographic Experts Group
`size`: 4106   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/55.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/55.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/55.png
`format`: Joint Photographic Experts Group
`size`: 2824   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/196.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/196.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/196.png
`format`: Joint Photographic Experts Group
`size`: 12626   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/40.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/40.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/40.png
`format`: Joint Photographic Experts Group
`size`: 2073   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/87.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/87.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/87.png
`format`: Joint Photographic Experts Group
`size`: 4520   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/50.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/50.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/50.png
`format`: Joint Photographic Experts Group
`size`: 2662   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/92.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/92.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/92.png
`format`: Joint Photographic Experts Group
`size`: 4877   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/144.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/144.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/144.png
`format`: Joint Photographic Experts Group
`size`: 7964   




### ios/Runner/Assets.xcassets/AppIcon.appiconset/152.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Assets.xcassets/AppIcon.appiconset/152.png
`relative_path`: ios/Runner/Assets.xcassets/AppIcon.appiconset/152.png
`format`: Joint Photographic Experts Group
`size`: 8869   




### ios/Runner/Base.lproj/LaunchScreen.storyboard
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Base.lproj/LaunchScreen.storyboard
`relative_path`: ios/Runner/Base.lproj/LaunchScreen.storyboard
`format`: Extensible Markup Language
`size`: 2377   




### ios/Runner/Base.lproj/Main.storyboard
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Base.lproj/Main.storyboard
`relative_path`: ios/Runner/Base.lproj/Main.storyboard
`format`: Extensible Markup Language
`size`: 1605   




### ios/Runner/AppDelegate.swift
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/AppDelegate.swift
`relative_path`: ios/Runner/AppDelegate.swift
`format`: Arbitrary Binary Data
`size`: 521   


``````
import Flutter
import UIKit
import GoogleMaps

@main
@objc class AppDelegate: FlutterAppDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    GMSServices.provideAPIKey("AIzaSyCoSRCEErXy_62KTBJZfCYzDNiGuLm89oQ") // Füge hier den API-Schlüssel hinzu
    GeneratedPluginRegistrant.register(with: self)
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
}

``````


### ios/Runner/Info.plist
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner/Info.plist
`relative_path`: ios/Runner/Info.plist
`format`: Extensible Markup Language
`size`: 2941   




### ios/Runner.xcodeproj/project.pbxproj
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner.xcodeproj/project.pbxproj
`relative_path`: ios/Runner.xcodeproj/project.pbxproj
`format`: Arbitrary Binary Data
`size`: 31440   




### ios/Runner.xcodeproj/project.xcworkspace/contents.xcworkspacedata
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner.xcodeproj/project.xcworkspace/contents.xcworkspacedata
`relative_path`: ios/Runner.xcodeproj/project.xcworkspace/contents.xcworkspacedata
`format`: Extensible Markup Language
`size`: 135   




### ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`relative_path`: ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/IDEWorkspaceChecks.plist
`format`: Extensible Markup Language
`size`: 238   




### ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`relative_path`: ios/Runner.xcodeproj/project.xcworkspace/xcshareddata/WorkspaceSettings.xcsettings
`format`: Extensible Markup Language
`size`: 226   




### ios/Runner.xcodeproj/xcshareddata/xcschemes/Runner.xcscheme
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Runner.xcodeproj/xcshareddata/xcschemes/Runner.xcscheme
`relative_path`: ios/Runner.xcodeproj/xcshareddata/xcschemes/Runner.xcscheme
`format`: Extensible Markup Language
`size`: 3647   




### ios/Flutter/Debug.xcconfig
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Flutter/Debug.xcconfig
`relative_path`: ios/Flutter/Debug.xcconfig
`format`: Arbitrary Binary Data
`size`: 107   


``````
#include? "Pods/Target Support Files/Pods-Runner/Pods-Runner.debug.xcconfig"
#include "Generated.xcconfig"

``````


### ios/Flutter/Release.xcconfig
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Flutter/Release.xcconfig
`relative_path`: ios/Flutter/Release.xcconfig
`format`: Arbitrary Binary Data
`size`: 109   


``````
#include? "Pods/Target Support Files/Pods-Runner/Pods-Runner.release.xcconfig"
#include "Generated.xcconfig"

``````


### ios/Flutter/AppFrameworkInfo.plist
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Flutter/AppFrameworkInfo.plist
`relative_path`: ios/Flutter/AppFrameworkInfo.plist
`format`: Extensible Markup Language
`size`: 774   




### ios/Podfile
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Podfile
`relative_path`: ios/Podfile
`format`: Arbitrary Binary Data
`size`: 1412   


``````
# Uncomment this line to define a global platform for your project
platform :ios, '14.0'

# CocoaPods analytics sends network stats synchronously affecting flutter build latency.
ENV['COCOAPODS_DISABLE_STATS'] = 'true'

project 'Runner', {
  'Debug' => :debug,
  'Profile' => :release,
  'Release' => :release,
}

def flutter_root
  generated_xcode_build_settings_path = File.expand_path(File.join('..', 'Flutter', 'Generated.xcconfig'), __FILE__)
  unless File.exist?(generated_xcode_build_settings_path)
    raise "#{generated_xcode_build_settings_path} must exist. If you're running pod install manually, make sure flutter pub get is executed first"
  end

  File.foreach(generated_xcode_build_settings_path) do |line|
    matches = line.match(/FLUTTER_ROOT\=(.*)/)
    return matches[1].strip if matches
  end
  raise "FLUTTER_ROOT not found in #{generated_xcode_build_settings_path}. Try deleting Generated.xcconfig, then run flutter pub get"
end

require File.expand_path(File.join('packages', 'flutter_tools', 'bin', 'podhelper'), flutter_root)

flutter_ios_podfile_setup

target 'Runner' do
  use_frameworks!
  use_modular_headers!

  flutter_install_all_ios_pods File.dirname(File.realpath(__FILE__))
  target 'RunnerTests' do
    inherit! :search_paths
  end
end

post_install do |installer|
  installer.pods_project.targets.each do |target|
    flutter_additional_ios_build_settings(target)
  end
end

``````


### ios/Podfile.lock
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/ios/Podfile.lock
`relative_path`: ios/Podfile.lock
`format`: Arbitrary Binary Data
`size`: 8813   




### README.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/README.md
`relative_path`: README.md
`format`: Arbitrary Binary Data
`size`: 558   


``````
# vacuul_reworked

A new Flutter project.

## Getting Started

This project is a starting point for a Flutter application.

A few resources to get you started if this is your first Flutter project:

- [Lab: Write your first Flutter app](https://docs.flutter.dev/get-started/codelab)
- [Cookbook: Useful Flutter samples](https://docs.flutter.dev/cookbook)

For help getting started with Flutter development, view the
[online documentation](https://docs.flutter.dev/), which offers tutorials,
samples, guidance on mobile development, and a full API reference.

``````


### pubspec.yaml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/pubspec.yaml
`relative_path`: pubspec.yaml
`format`: Arbitrary Binary Data
`size`: 7204   




### android/app/build.gradle
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/build.gradle
`relative_path`: android/app/build.gradle
`format`: Arbitrary Binary Data
`size`: 1368   


``````
plugins {
    id "com.android.application"
    id "kotlin-android"
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id "dev.flutter.flutter-gradle-plugin"
}

android {
    namespace = "com.vacuul.health"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_1_8
    }

    defaultConfig {
        // TODO: Specify your own unique Application ID (https://developer.android.com/studio/build/application-id.html).
        applicationId = "com.example.vacuul_reworked"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    buildTypes {
        release {
            // TODO: Add your own signing config for the release build.
            // Signing with the debug keys for now, so `flutter run --release` works.
            signingConfig = signingConfigs.debug
        }
    }
}

flutter {
    source = "../.."
}

``````


### android/app/src/profile/AndroidManifest.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/profile/AndroidManifest.xml
`relative_path`: android/app/src/profile/AndroidManifest.xml
`format`: Arbitrary Binary Data
`size`: 378   


``````
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <!-- The INTERNET permission is required for development. Specifically,
         the Flutter tool needs it to communicate with the running application
         to allow setting breakpoints, to provide hot reload, etc.
    -->
    <uses-permission android:name="android.permission.INTERNET"/>
</manifest>

``````


### android/app/src/main/res/mipmap-mdpi/ic_launcher.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/main/res/mipmap-mdpi/ic_launcher.png
`relative_path`: android/app/src/main/res/mipmap-mdpi/ic_launcher.png
`format`: Portable Network Graphics
`size`: 982   




### android/app/src/main/res/mipmap-hdpi/ic_launcher.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/main/res/mipmap-hdpi/ic_launcher.png
`relative_path`: android/app/src/main/res/mipmap-hdpi/ic_launcher.png
`format`: Portable Network Graphics
`size`: 1501   




### android/app/src/main/res/drawable/launch_background.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/main/res/drawable/launch_background.xml
`relative_path`: android/app/src/main/res/drawable/launch_background.xml
`format`: Extensible Markup Language
`size`: 434   




### android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png
`relative_path`: android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png
`format`: Portable Network Graphics
`size`: 4380   




### android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png
`relative_path`: android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png
`format`: Portable Network Graphics
`size`: 3111   




### android/app/src/main/res/values-night/styles.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/main/res/values-night/styles.xml
`relative_path`: android/app/src/main/res/values-night/styles.xml
`format`: Extensible Markup Language
`size`: 995   




### android/app/src/main/res/values/styles.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/main/res/values/styles.xml
`relative_path`: android/app/src/main/res/values/styles.xml
`format`: Extensible Markup Language
`size`: 996   




### android/app/src/main/res/drawable-v21/launch_background.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/main/res/drawable-v21/launch_background.xml
`relative_path`: android/app/src/main/res/drawable-v21/launch_background.xml
`format`: Extensible Markup Language
`size`: 438   




### android/app/src/main/res/mipmap-xhdpi/ic_launcher.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/main/res/mipmap-xhdpi/ic_launcher.png
`relative_path`: android/app/src/main/res/mipmap-xhdpi/ic_launcher.png
`format`: Portable Network Graphics
`size`: 2000   




### android/app/src/main/AndroidManifest.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/main/AndroidManifest.xml
`relative_path`: android/app/src/main/AndroidManifest.xml
`format`: Arbitrary Binary Data
`size`: 2417   


``````
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application
        android:label="vacuul_reworked"
        android:name="${applicationName}"
        android:icon="@mipmap/ic_launcher">
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:launchMode="singleTop"
            android:taskAffinity=""
            android:theme="@style/LaunchTheme"
            android:configChanges="orientation|keyboardHidden|keyboard|screenSize|smallestScreenSize|locale|layoutDirection|fontScale|screenLayout|density|uiMode"
            android:hardwareAccelerated="true"
            android:windowSoftInputMode="adjustResize">
            <!-- Specifies an Android theme to apply to this Activity as soon as
                 the Android process has started. This theme is visible to the user
                 while the Flutter UI initializes. After that, this theme continues
                 to determine the Window background behind the Flutter UI. -->
            <meta-data
              android:name="io.flutter.embedding.android.NormalTheme"
              android:resource="@style/NormalTheme"
              />
            <intent-filter>
                <action android:name="android.intent.action.MAIN"/>
                <category android:name="android.intent.category.LAUNCHER"/>
            </intent-filter>
        </activity>
        <!-- Don't delete the meta-data below.
             This is used by the Flutter tool to generate GeneratedPluginRegistrant.java -->
        <meta-data
            android:name="flutterEmbedding"
            android:value="2" />

        <meta-data
            android:name="com.google.android.geo.API_KEY"
            android:value="AIzaSyCoSRCEErXy_62KTBJZfCYzDNiGuLm89oQ" />
    </application>
    <!-- Required to query activities that can process text, see:
         https://developer.android.com/training/package-visibility and
         https://developer.android.com/reference/android/content/Intent#ACTION_PROCESS_TEXT.

         In particular, this is used by the Flutter engine in io.flutter.plugin.text.ProcessTextPlugin. -->
    <queries>
        <intent>
            <action android:name="android.intent.action.PROCESS_TEXT"/>
            <data android:mimeType="text/plain"/>
        </intent>
    </queries>
    <uses-permission android:name="android.permission.CAMERA" />
</manifest>

``````


### android/app/src/main/java/com/example/vacuul_reworked/MainActivity.java
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/main/java/com/example/vacuul_reworked/MainActivity.java
`relative_path`: android/app/src/main/java/com/example/vacuul_reworked/MainActivity.java
`format`: Arbitrary Binary Data
`size`: 146   


``````
package com.example.vacuul_reworked;

import io.flutter.embedding.android.FlutterActivity;

public class MainActivity extends FlutterActivity {
}

``````


### android/app/src/debug/AndroidManifest.xml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/app/src/debug/AndroidManifest.xml
`relative_path`: android/app/src/debug/AndroidManifest.xml
`format`: Arbitrary Binary Data
`size`: 378   


``````
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <!-- The INTERNET permission is required for development. Specifically,
         the Flutter tool needs it to communicate with the running application
         to allow setting breakpoints, to provide hot reload, etc.
    -->
    <uses-permission android:name="android.permission.INTERNET"/>
</manifest>

``````


### android/gradle/wrapper/gradle-wrapper.properties
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/gradle/wrapper/gradle-wrapper.properties
`relative_path`: android/gradle/wrapper/gradle-wrapper.properties
`format`: Arbitrary Binary Data
`size`: 200   


``````
distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-8.3-all.zip

``````


### android/build.gradle
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/build.gradle
`relative_path`: android/build.gradle
`format`: Arbitrary Binary Data
`size`: 322   


``````
allprojects {
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.buildDir = "../build"
subprojects {
    project.buildDir = "${rootProject.buildDir}/${project.name}"
}
subprojects {
    project.evaluationDependsOn(":app")
}

tasks.register("clean", Delete) {
    delete rootProject.buildDir
}

``````


### android/gradle.properties
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/gradle.properties
`relative_path`: android/gradle.properties
`format`: Arbitrary Binary Data
`size`: 135   


``````
org.gradle.jvmargs=-Xmx4G -XX:MaxMetaspaceSize=2G -XX:+HeapDumpOnOutOfMemoryError
android.useAndroidX=true
android.enableJetifier=true

``````


### android/settings.gradle
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/android/settings.gradle
`relative_path`: android/settings.gradle
`format`: Arbitrary Binary Data
`size`: 727   


``````
pluginManagement {
    def flutterSdkPath = {
        def properties = new Properties()
        file("local.properties").withInputStream { properties.load(it) }
        def flutterSdkPath = properties.getProperty("flutter.sdk")
        assert flutterSdkPath != null, "flutter.sdk not set in local.properties"
        return flutterSdkPath
    }()

    includeBuild("$flutterSdkPath/packages/flutter_tools/gradle")

    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

plugins {
    id "dev.flutter.flutter-plugin-loader" version "1.0.0"
    id "com.android.application" version "8.1.0" apply false
    id "org.jetbrains.kotlin.android" version "1.8.22" apply false
}

include ":app"

``````


### lib/screens/booking/MachineCard.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/booking/MachineCard.dart
`relative_path`: lib/screens/booking/MachineCard.dart
`format`: Arbitrary Binary Data
`size`: 3221   


``````
import 'package:flutter/material.dart';

class MachineCard extends StatelessWidget {
  final String id;
  final String location;
  final VoidCallback onSelect;

  const MachineCard({
    super.key,
    required this.id,
    required this.location,
    required this.onSelect,
  });

  @override
  Widget build(BuildContext context) {
    final String shortId = id.length >= 3 ? id.substring(id.length - 3) : id;

    return Padding(
      padding: const EdgeInsets.all(16.0),
      child: Container(
        height: 166.0,
        width: double.infinity,
        decoration: BoxDecoration(
          color: Colors.white,
          borderRadius: BorderRadius.circular(4.0),
          boxShadow: [
            BoxShadow(
              color: Colors.grey.withOpacity(0.1),
              spreadRadius: 1,
              blurRadius: 3,
              offset: const Offset(0, 1),
            ),
          ],
        ),
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Device-ID (letzte 3 Stellen)
              Text(
                "Device $shortId",
                style: const TextStyle(
                  fontSize: 18.0,
                  fontFamily: 'IBM Plex Mono',
                  fontWeight: FontWeight.w600,
                  color: Color.fromRGBO(28, 27, 31, 1),
                ),
              ),
              const SizedBox(height: 8.0),

              // Location-Text
              Text(
                location,
                style: const TextStyle(
                  fontSize: 16.0,
                  fontFamily: 'Abel',
                  color: Color.fromRGBO(81, 73, 75, 1),
                ),
              ),
              const SizedBox(height: 8.0),

              // SELECT-Button (expandiert und zentriert)
              Expanded(
                child: Align(
                  alignment: Alignment.centerLeft,
                  child: SizedBox(
                    width: double.infinity, // Button expandiert auf volle Breite
                    child: ElevatedButton(
                      onPressed: onSelect,
                      style: ElevatedButton.styleFrom(
                        backgroundColor: const Color.fromRGBO(197, 199, 197, 1),
                        padding: const EdgeInsets.symmetric(
                            vertical: 12.0, horizontal: 20.0),
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(4.0),
                        ),
                      ),
                      child: const Text(
                        "SELECT",
                        textAlign: TextAlign.center, // Zentriert den Text
                        style: TextStyle(
                          fontSize: 14.0,
                          fontFamily: 'IBM Plex Mono',
                          fontWeight: FontWeight.w500,
                          color: Color.fromRGBO(28, 27, 31, 1),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

``````


### lib/screens/booking/new_appointment.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/booking/new_appointment.dart
`relative_path`: lib/screens/booking/new_appointment.dart
`format`: Arbitrary Binary Data
`size`: 10284   




### lib/screens/booking/map_content.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/booking/map_content.dart
`relative_path`: lib/screens/booking/map_content.dart
`format`: Arbitrary Binary Data
`size`: 11015   




### lib/screens/booking/TimeSlotSelectionModal.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/booking/TimeSlotSelectionModal.dart
`relative_path`: lib/screens/booking/TimeSlotSelectionModal.dart
`format`: Arbitrary Binary Data
`size`: 10667   




### lib/screens/booking/book_appointment.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/booking/book_appointment.dart
`relative_path`: lib/screens/booking/book_appointment.dart
`format`: Arbitrary Binary Data
`size`: 34537   




### lib/screens/booking/list_content.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/booking/list_content.dart
`relative_path`: lib/screens/booking/list_content.dart
`format`: Arbitrary Binary Data
`size`: 2215   


``````
import 'package:flutter/material.dart';
import 'package:Vacuul/screens/globals.dart';

import 'MachineCard.dart';
import 'TimeSlotSelectionModal.dart';

class ListContent extends StatefulWidget {
  final List<dynamic> machines;

  const ListContent({super.key, required this.machines});

  @override
  State<ListContent> createState() => _ListContentState();
}

class _ListContentState extends State<ListContent> with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late List<Animation<Offset>> _animations;

  @override
  void initState() {
    super.initState();

    _controller = AnimationController(
      duration: const Duration(seconds: 2), // Gesamtdauer der Animation
      vsync: this,
    );

    // Erstelle für jede Maschine eine Animation
    _animations = List.generate(widget.machines.length, (index) {
      final delay = index * 0.2; // Verzögerung zwischen den Einflügen
      final start = delay / 2; // Startpunkt für den Tween
      final end = (delay + 0.5) / 2; // Endpunkt für den Tween
      return Tween<Offset>(
        begin: const Offset(1.0, 0.0), // Rechts starten
        end: Offset.zero, // In die Zielposition bewegen
      ).animate(
        CurvedAnimation(
          parent: _controller,
          curve: Interval(start, end, curve: Curves.easeOut), // Verzögerung und Animation
        ),
      );
    });

    // Animation starten
    _controller.forward();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      itemCount: widget.machines.length,
      itemBuilder: (context, index) {
        final machine = widget.machines[index];
        return SlideTransition(
          position: _animations[index], // Animation für dieses Item
          child: MachineCard(
            id: machine['id'].toString(), // ID der Maschine
            location: machine['location'] ?? 'Unknown Location', // Standort
            onSelect: () {
              selectMachineID = machine['id'].toString();
              TimeSlotSelectionModal.show(context); // Modal anzeigen
            },
          ),
        );
      },
    );
  }
}

``````


### lib/screens/payment/scan_qr.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/payment/scan_qr.dart
`relative_path`: lib/screens/payment/scan_qr.dart
`format`: Arbitrary Binary Data
`size`: 3163   


``````
import 'package:Vacuul/screens/payment/unlockMachine.dart';
import 'package:Vacuul/widgets/top_bar.dart';
import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

class ScanQr extends StatefulWidget {
  final Map<String, dynamic> booking;

  const ScanQr({super.key, required this.booking});

  @override
  State<ScanQr> createState() => _ScanQrState();
}

class _ScanQrState extends State<ScanQr> {
  bool _hasScanned = false;

  @override
  Widget build(BuildContext context) {
    final String machineId = widget.booking['machineId'] ?? "Unknown";

    return Scaffold(
      backgroundColor: const Color.fromRGBO(198, 204, 198, 1),
      body: SafeArea(
        child: Column(
          children: [
            const TopBar(
              title: "SCAN QR CODE",
              cancelButton: true,
            ),
            Expanded(
              child: Stack(
                children: [
                  MobileScanner(
                      onDetect: (BarcodeCapture capture) {
                        if (_hasScanned) return;

                        setState(() {
                          _hasScanned = true;
                        });

                        final List<Barcode> barcodes = capture.barcodes;
                        for (final barcode in barcodes) {
                          if (barcode.rawValue != null) {
                            final code = barcode.rawValue!;
                            debugPrint('QR Code found: $code');

                            Navigator.pop(context);
                            Navigator.push(
                              context,
                              MaterialPageRoute(
                                builder: (context) => UnlockMachine(booking: widget.booking),
                              ),
                            );
                            break;
                          }
                        }
                      }
                  ),
                  Center(
                    child: Container(
                      width: 250,
                      height: 250,
                      decoration: BoxDecoration(
                        border: Border.all(color: Colors.white, width: 2),
                        borderRadius: BorderRadius.circular(8),
                        color: Colors.transparent,
                      ),
                    ),
                  ),
                  Positioned(
                    top: MediaQuery.of(context).size.height * 0.2,
                    left: 16,
                    right: 16,
                    child: Text(
                      "Scan QR code for device $machineId to activate it",
                      textAlign: TextAlign.center,
                      style: const TextStyle(
                        fontFamily: 'Abel',
                        fontSize: 16,
                        fontWeight: FontWeight.w400,
                        height: 1.25,
                        color: Colors.white,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

``````


### lib/screens/payment/unlockMachine.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/payment/unlockMachine.dart
`relative_path`: lib/screens/payment/unlockMachine.dart
`format`: Arbitrary Binary Data
`size`: 15388   




### lib/screens/payment/demoStartMachine.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/payment/demoStartMachine.dart
`relative_path`: lib/screens/payment/demoStartMachine.dart
`format`: Arbitrary Binary Data
`size`: 5808   




### lib/screens/payment/approve.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/payment/approve.dart
`relative_path`: lib/screens/payment/approve.dart
`format`: Arbitrary Binary Data
`size`: 1888   


``````
import 'package:Vacuul/screens/globals.dart';
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:Vacuul/screens/authentication/appwrite_service.dart';
import 'package:Vacuul/widgets/top_bar.dart';

class Approve extends StatefulWidget {
  final String appointmentID;
  final Map<String, dynamic> therapySettings;
  final paymentIntent;
  const Approve({
    super.key,
    required this.appointmentID,
    required this.therapySettings,
    required this.paymentIntent,
  });


  @override
  State<Approve> createState() => _ApproveState();
}

class _ApproveState extends State<Approve> {
  bool _showLoader = true;

  @override
  void initState() {
    super.initState();
    Future.delayed(const Duration(seconds: 0), () async {
      await AppWriteService().endBooking(
        widget.appointmentID,
        widget.therapySettings,
        widget.paymentIntent
      );
      await AppWriteService().getBookings();
      setState(() {
        _showLoader = false;
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: const Color.fromRGBO(198, 204, 198, 1),
      body: SafeArea(
        child: Column(
          children: [
            const TopBar(title: "BOOKING CONFIRMED", cancelButton: true,),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Center(
                  child: _showLoader
                      ? const CircularProgressIndicator(
                    color: Color.fromRGBO(28, 27, 31, 1),
                  )
                      : SvgPicture.asset(
                    "lib/assets/svg/scren.svg", // Pfad zum SVG
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

``````


### lib/screens/profile/feedback.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/profile/feedback.dart
`relative_path`: lib/screens/profile/feedback.dart
`format`: Arbitrary Binary Data
`size`: 6334   




### lib/screens/profile/profile.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/profile/profile.dart
`relative_path`: lib/screens/profile/profile.dart
`format`: Arbitrary Binary Data
`size`: 23884   




### lib/screens/profile/helpcenter.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/profile/helpcenter.dart
`relative_path`: lib/screens/profile/helpcenter.dart
`format`: Arbitrary Binary Data
`size`: 8371   




### lib/screens/profile/profile_settings.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/profile/profile_settings.dart
`relative_path`: lib/screens/profile/profile_settings.dart
`format`: Arbitrary Binary Data
`size`: 15369   




### lib/screens/globals.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/globals.dart
`relative_path`: lib/screens/globals.dart
`format`: Arbitrary Binary Data
`size`: 285   


``````
import 'package:flutter/material.dart';

String? globalPickedDate;

String? selectedTimeSlotGlobal;

String? selectMachineID;

String? currentPaymentIntent;

String? userBookings;

String globalSelectedLight = "Blue";

int globalSelectedTemperature = 7;

int globalSelectedRounds = 3;

``````


### lib/screens/main/home.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/main/home.dart
`relative_path`: lib/screens/main/home.dart
`format`: Arbitrary Binary Data
`size`: 20476   




### lib/screens/main/bookings.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/main/bookings.dart
`relative_path`: lib/screens/main/bookings.dart
`format`: Arbitrary Binary Data
`size`: 9876   




### lib/screens/main/reading.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/main/reading.dart
`relative_path`: lib/screens/main/reading.dart
`format`: Arbitrary Binary Data
`size`: 10957   




### lib/screens/main/reaading_detail.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/main/reaading_detail.dart
`relative_path`: lib/screens/main/reaading_detail.dart
`format`: Arbitrary Binary Data
`size`: 10868   




### lib/screens/authentication/appwrite_service.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/authentication/appwrite_service.dart
`relative_path`: lib/screens/authentication/appwrite_service.dart
`format`: Arbitrary Binary Data
`size`: 8225   




### lib/screens/authentication/forgot_password.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/authentication/forgot_password.dart
`relative_path`: lib/screens/authentication/forgot_password.dart
`format`: Arbitrary Binary Data
`size`: 3241   


``````
import 'package:flutter/material.dart';
import 'package:Vacuul/screens/authentication/text_field_auth.dart';
import 'package:Vacuul/widgets/top_bar.dart';

class ForgotPassword extends StatefulWidget {
  const ForgotPassword({super.key});

  @override
  State<ForgotPassword> createState() => _ForgotPasswordState();
}

class _ForgotPasswordState extends State<ForgotPassword> {
  final TextEditingController _emailController = TextEditingController();

  bool _isValidEmail(String email) {
    return email.isNotEmpty && email.length > 5 && email.contains('@');
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: const Color.fromRGBO(198, 204, 198, 1),
      body: SafeArea(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            const TopBar(title: "FORGOT PASSWORD", backButton: true),
            Expanded(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  TextFieldAuth(
                    controller: _emailController,
                    hintText: "EMAIL",
                    isPassword: false,
                  ),
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 12.0),
                    child: SizedBox(
                      height: 48.0,
                      width: double.infinity,
                      child: ElevatedButton(
                        onPressed: () {
                          final email = _emailController.text.trim();

                          if (!_isValidEmail(email)) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(content: Text('Bitte geben Sie eine gültige E-Mail-Adresse ein.')),
                            );
                            return;
                          }

                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(content: Text('Password Reset Instructions have been sent out to your E-Mail.')),
                          );
                          Navigator.of(context).pop();
                        },
                        style: ElevatedButton.styleFrom(
                          backgroundColor: const Color.fromRGBO(28, 27, 31, 1),
                          shape: RoundedRectangleBorder(
                            borderRadius: BorderRadius.circular(8.0),
                          ),
                        ),
                        child: const Text(
                          "SEND RESET LINK",
                          style: TextStyle(
                            fontSize: 14.0,
                            fontWeight: FontWeight.w500,
                            fontFamily: 'IBM Plex Mono',
                            color: Colors.white,
                          ),
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            )
          ],
        ),
      ),
    );
  }
}

``````


### lib/screens/authentication/sign_in_page.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/authentication/sign_in_page.dart
`relative_path`: lib/screens/authentication/sign_in_page.dart
`format`: Arbitrary Binary Data
`size`: 14470   




### lib/screens/authentication/sign_up_page.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/authentication/sign_up_page.dart
`relative_path`: lib/screens/authentication/sign_up_page.dart
`format`: Arbitrary Binary Data
`size`: 13936   




### lib/screens/authentication/text_field_auth.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/screens/authentication/text_field_auth.dart
`relative_path`: lib/screens/authentication/text_field_auth.dart
`format`: Arbitrary Binary Data
`size`: 2004   


``````
import 'package:flutter/material.dart';

class TextFieldAuth extends StatelessWidget {
  final String hintText;
  final bool isPassword;
  final TextEditingController? controller;

  const TextFieldAuth({
    super.key,
    required this.hintText,
    required this.isPassword,
    this.controller,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
      child: SizedBox(
        height: 54.0,
        child: TextField(
          controller: controller,
          obscureText: isPassword,
          style: const TextStyle(
            fontSize: 16.0,
            color: Color.fromRGBO(28, 27, 31, 1),
            fontWeight: FontWeight.w400,
          ),
          decoration: InputDecoration(
            labelText: hintText,
            labelStyle: const TextStyle(
              fontSize: 10.0,
              color: Color.fromRGBO(81, 73, 75, 1),
              fontWeight: FontWeight.w600,
            ),
            filled: true,
            fillColor: const Color(0xFFFFFFFF),
            contentPadding: const EdgeInsets.symmetric(
              vertical: 8.0,
              horizontal: 16.0,
            ),
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8.0),
              borderSide: const BorderSide(
                color: Color(0xFF9BA19B),
                width: 1.0,
              ),
            ),
            enabledBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8.0),
              borderSide: const BorderSide(
                color: Color(0xFF9BA19B),
                width: 1.0,
              ),
            ),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8.0),
              borderSide: const BorderSide(
                color: Color(0xFF9BA19B),
                width: 1.0,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

``````


### lib/main.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/main.dart
`relative_path`: lib/main.dart
`format`: Arbitrary Binary Data
`size`: 1746   


``````
import 'package:Vacuul/screens/profile/feedback.dart';
import 'package:Vacuul/screens/profile/helpcenter.dart';
import 'package:Vacuul/screens/profile/profile_settings.dart';
import 'package:flutter/material.dart';
import 'package:Vacuul/screens/authentication/forgot_password.dart';
import 'package:Vacuul/screens/booking/book_appointment.dart';
import 'package:Vacuul/screens/booking/new_appointment.dart';
import 'package:Vacuul/screens/main/bookings.dart';
import 'package:Vacuul/screens/payment/approve.dart';
import 'package:Vacuul/screens/profile/profile.dart';
import 'SplashScreen.dart';
import 'screens/authentication/sign_up_page.dart';
import 'screens/authentication/sign_in_page.dart';
import 'screens/main/home.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter App',
      initialRoute: '/splash',
      routes: {
        '/splash': (context) => const SplashScreen(),
        '/signup': (context) => const SignUpPage(),
        '/signin': (context) => const SignInPage(),
        '/booking': (context) => const Bookings(),
        '/home': (context) => const HomeScreen(),
        '/forgot-password': (context) => const ForgotPassword(),
        '/newAppointment': (context) => const NewAppointment(),
        '/bookAppointment': (context) => const BookAppointment(),
        '/profile': (context) => const Profile(),
        '/profileSettings': (context) => const ProfileSettings(),
        '/feedback': (context) => const RateFeedback(),
        '/helpcenter': (context) => const Helpcenter()
      },
    );
  }
}

``````


### lib/SplashScreen.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/SplashScreen.dart
`relative_path`: lib/SplashScreen.dart
`format`: Arbitrary Binary Data
`size`: 4632   


``````
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_stripe/flutter_stripe.dart';
import 'package:flutter_svg/svg.dart';
import 'package:lottie/lottie.dart';
import 'screens/authentication/appwrite_service.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'dart:ui' as ui;

class SplashScreen extends StatefulWidget {
  const SplashScreen({super.key});

  @override
  State<SplashScreen> createState() => _SplashScreenState();
}

class _SplashScreenState extends State<SplashScreen> {
  final AppWriteService appWriteService = AppWriteService();

  @override
  void initState() {
    super.initState();
    _checkSession();
  }

  Future<void> _loadNotificationPreference() async {
    final prefs = await SharedPreferences.getInstance();
    if (prefs.containsKey('selected_language')) {
      return; // Sprachpräferenz ist bereits vorhanden, keine Aktion erforderlich
    }
    final notificationsEnabled = prefs.setString('notifications_enabled', "OFF");
  }


  Future<void> checkAndSetLanguagePreference() async {
    // Stellt sicher, dass der Flutter-Binding initialisiert ist
    WidgetsFlutterBinding.ensureInitialized();

    final prefs = await SharedPreferences.getInstance();

    // Überprüfen, ob die Sprachpräferenz bereits gespeichert ist
    if (prefs.containsKey('selected_language')) {
      return; // Sprachpräferenz ist bereits vorhanden, keine Aktion erforderlich
    }

    // Systemsprache ermitteln
    String systemLanguage = Platform.localeName.split('_').first.toLowerCase();

    // Mapping von Systemsprache zu unterstütztem Format
    Map<String, String> languageMapping = {
      'de': 'GERMAN',
      'en': 'ENGLISH',
      'fr': 'FRENCH',
      'es': 'SPANISH',
      'sw': 'SWAHILI',
    };

    // Standard auf Englisch setzen, falls Systemsprache nicht unterstützt wird
    String selectedLanguage = languageMapping[systemLanguage] ?? 'ENGLISH';

    // Sprachpräferenz speichern
    await prefs.setString('selected_language', selectedLanguage);
    print("Language preference set to: $selectedLanguage"); // Debugging
  }


  Future<void> _checkSession() async {
    Stripe.publishableKey = 'pk_test_51QfeeRE9t48iT7vtu8s8N5NxSOjuanOvluEpqwmQdURGyeAl0BRtkOtE9cE2tUjhxWpAXd0j5yCCY9YwmYiqFKJW00GyVtGjQ4';
    Stripe.instance.applySettings();
    final isLoggedIn = await appWriteService.isUserLoggedIn();
    await checkAndSetLanguagePreference();
    await _loadNotificationPreference();

    if (isLoggedIn) {
      await appWriteService.getBookings();
      Navigator.pushReplacementNamed(context, '/home');
    } else {
      Navigator.pushReplacementNamed(context, '/signup');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      body: Stack(
        alignment: Alignment.bottomCenter,
        children: [
          Positioned(top: 100,
            child: SvgPicture.asset(
              'lib/assets/svg/vacuul.svg',
              width: 200, // Größe des Logos
            ),),
          // Lottie-Animation am unteren Rand
          Positioned(
            bottom: 0,
            child: Lottie.asset(
              'lib/assets/animations/dots.json',
              width: MediaQuery.of(context).size.width,
              height: MediaQuery.of(context).size.height / 1.3,
              fit: BoxFit.cover,
            ),
          ),
          // Logo und Text
          Positioned(
            bottom: MediaQuery.of(context).size.height / 2 - 350, // Über der Lottie-Animation
            left: 16, // Abstand vom linken Rand
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                SvgPicture.asset(
                  'lib/assets/svg/vacuul_logo.svg',
                  width: 100, // Größe des Logos
                ),
                const SizedBox(height: 16),
                const Text(
                  "UNLEASH THE\nPOWER IN YOU",
                  style: TextStyle(
                      fontFamily: 'Abel',
                      fontSize: 56,
                      fontWeight: FontWeight.w400,
                      height: 64 / 56,
                      decoration: TextDecoration.underline,
                      decorationStyle: TextDecorationStyle.solid,
                      decorationColor: Colors.white,
                      decorationThickness: 1.5,
                      color: Colors.white
                  ),
                  textAlign: TextAlign.left,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

``````


### lib/assets/svg/checkmark.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/checkmark.svg
`relative_path`: lib/assets/svg/checkmark.svg
`format`: Scalable Vector Graphics
`size`: 245   




### lib/assets/svg/bulb.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/bulb.svg
`relative_path`: lib/assets/svg/bulb.svg
`format`: Scalable Vector Graphics
`size`: 919   




### lib/assets/svg/temperature.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/temperature.svg
`relative_path`: lib/assets/svg/temperature.svg
`format`: Scalable Vector Graphics
`size`: 811   




### lib/assets/svg/vacuul_logo.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/vacuul_logo.svg
`relative_path`: lib/assets/svg/vacuul_logo.svg
`format`: Scalable Vector Graphics
`size`: 280   




### lib/assets/svg/home.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/home.svg
`relative_path`: lib/assets/svg/home.svg
`format`: Scalable Vector Graphics
`size`: 885   




### lib/assets/svg/round.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/round.svg
`relative_path`: lib/assets/svg/round.svg
`format`: Scalable Vector Graphics
`size`: 867   




### lib/assets/svg/readingunselected.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/readingunselected.svg
`relative_path`: lib/assets/svg/readingunselected.svg
`format`: Scalable Vector Graphics
`size`: 445   




### lib/assets/svg/expand.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/expand.svg
`relative_path`: lib/assets/svg/expand.svg
`format`: Scalable Vector Graphics
`size`: 188   




### lib/assets/svg/vacuul.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/vacuul.svg
`relative_path`: lib/assets/svg/vacuul.svg
`format`: Scalable Vector Graphics
`size`: 4049   




### lib/assets/svg/mastercard.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/mastercard.svg
`relative_path`: lib/assets/svg/mastercard.svg
`format`: Scalable Vector Graphics
`size`: 516   




### lib/assets/svg/bookingsunselected.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/bookingsunselected.svg
`relative_path`: lib/assets/svg/bookingsunselected.svg
`format`: Scalable Vector Graphics
`size`: 625   




### lib/assets/svg/google.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/google.svg
`relative_path`: lib/assets/svg/google.svg
`format`: Scalable Vector Graphics
`size`: 1258   




### lib/assets/svg/homeunselected.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/homeunselected.svg
`relative_path`: lib/assets/svg/homeunselected.svg
`format`: Scalable Vector Graphics
`size`: 885   




### lib/assets/svg/next.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/next.svg
`relative_path`: lib/assets/svg/next.svg
`format`: Scalable Vector Graphics
`size`: 214   




### lib/assets/svg/reading.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/reading.svg
`relative_path`: lib/assets/svg/reading.svg
`format`: Scalable Vector Graphics
`size`: 448   




### lib/assets/svg/language.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/language.svg
`relative_path`: lib/assets/svg/language.svg
`format`: Scalable Vector Graphics
`size`: 1478   




### lib/assets/svg/profile.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/profile.svg
`relative_path`: lib/assets/svg/profile.svg
`format`: Scalable Vector Graphics
`size`: 584   




### lib/assets/svg/scren.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/scren.svg
`relative_path`: lib/assets/svg/scren.svg
`format`: Scalable Vector Graphics
`size`: 180912   




### lib/assets/svg/bookings.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/bookings.svg
`relative_path`: lib/assets/svg/bookings.svg
`format`: Scalable Vector Graphics
`size`: 628   




### lib/assets/svg/calendar.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/calendar.svg
`relative_path`: lib/assets/svg/calendar.svg
`format`: Scalable Vector Graphics
`size`: 623   




### lib/assets/svg/ellipse1.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/ellipse1.svg
`relative_path`: lib/assets/svg/ellipse1.svg
`format`: Scalable Vector Graphics
`size`: 570   




### lib/assets/svg/notifications.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/notifications.svg
`relative_path`: lib/assets/svg/notifications.svg
`format`: Scalable Vector Graphics
`size`: 884   




### lib/assets/svg/apple.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/apple.svg
`relative_path`: lib/assets/svg/apple.svg
`format`: Scalable Vector Graphics
`size`: 1607   




### lib/assets/svg/calendar_dark.svg
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/svg/calendar_dark.svg
`relative_path`: lib/assets/svg/calendar_dark.svg
`format`: Scalable Vector Graphics
`size`: 625   




### lib/assets/images/loader.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/loader.png
`relative_path`: lib/assets/images/loader.png
`format`: Portable Network Graphics
`size`: 398077   




### lib/assets/images/freedom.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/freedom.png
`relative_path`: lib/assets/images/freedom.png
`format`: Joint Photographic Experts Group
`size`: 288322   




### lib/assets/images/imageCard4.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/imageCard4.png
`relative_path`: lib/assets/images/imageCard4.png
`format`: Joint Photographic Experts Group
`size`: 16948   




### lib/assets/images/imageCard3.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/imageCard3.png
`relative_path`: lib/assets/images/imageCard3.png
`format`: Joint Photographic Experts Group
`size`: 16006   




### lib/assets/images/imageCard2.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/imageCard2.png
`relative_path`: lib/assets/images/imageCard2.png
`format`: Joint Photographic Experts Group
`size`: 30524   




### lib/assets/images/imageCard1.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/imageCard1.png
`relative_path`: lib/assets/images/imageCard1.png
`format`: Joint Photographic Experts Group
`size`: 45392   




### lib/assets/images/news1.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/news1.png
`relative_path`: lib/assets/images/news1.png
`format`: Joint Photographic Experts Group
`size`: 21881   




### lib/assets/images/news2.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/news2.png
`relative_path`: lib/assets/images/news2.png
`format`: Joint Photographic Experts Group
`size`: 40915   




### lib/assets/images/news3.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/news3.png
`relative_path`: lib/assets/images/news3.png
`format`: Joint Photographic Experts Group
`size`: 36407   




### lib/assets/images/readingSliderImage2.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/readingSliderImage2.png
`relative_path`: lib/assets/images/readingSliderImage2.png
`format`: Joint Photographic Experts Group
`size`: 35480   




### lib/assets/images/readingSliderImage3.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/readingSliderImage3.png
`relative_path`: lib/assets/images/readingSliderImage3.png
`format`: Joint Photographic Experts Group
`size`: 30343   




### lib/assets/images/readingSliderImage1.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/readingSliderImage1.png
`relative_path`: lib/assets/images/readingSliderImage1.png
`format`: Joint Photographic Experts Group
`size`: 42535   




### lib/assets/images/marker.png
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/images/marker.png
`relative_path`: lib/assets/images/marker.png
`format`: Portable Network Graphics
`size`: 718   




### lib/assets/fonts/IBMPlexMono-Thin.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-Thin.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-Thin.ttf
`format`: TrueType
`size`: 136076   




### lib/assets/fonts/IBMPlexMono-Bold.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-Bold.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-Bold.ttf
`format`: TrueType
`size`: 135932   




### lib/assets/fonts/IBMPlexMono-MediumItalic.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-MediumItalic.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-MediumItalic.ttf
`format`: TrueType
`size`: 142128   




### lib/assets/fonts/IBMPlexMono-BoldItalic.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-BoldItalic.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-BoldItalic.ttf
`format`: TrueType
`size`: 142636   




### lib/assets/fonts/IBMPlexMono-Medium.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-Medium.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-Medium.ttf
`format`: TrueType
`size`: 134880   




### lib/assets/fonts/IBMPlexMono-ExtraLight.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-ExtraLight.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-ExtraLight.ttf
`format`: TrueType
`size`: 134160   




### lib/assets/fonts/IBMPlexMono-ExtraLightItalic.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-ExtraLightItalic.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-ExtraLightItalic.ttf
`format`: TrueType
`size`: 142780   




### lib/assets/fonts/IBMPlexMono-ThinItalic.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-ThinItalic.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-ThinItalic.ttf
`format`: TrueType
`size`: 144240   




### lib/assets/fonts/IBMPlexMono-Regular.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-Regular.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-Regular.ttf
`format`: TrueType
`size`: 133720   




### lib/assets/fonts/OpenSauceSans-Black.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/OpenSauceSans-Black.ttf
`relative_path`: lib/assets/fonts/OpenSauceSans-Black.ttf
`format`: TrueType
`size`: 71180   




### lib/assets/fonts/IBMPlexMono-Light.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-Light.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-Light.ttf
`format`: TrueType
`size`: 133392   




### lib/assets/fonts/IBMPlexMono-Italic.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-Italic.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-Italic.ttf
`format`: TrueType
`size`: 142032   




### lib/assets/fonts/Abel-Regular.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/Abel-Regular.ttf
`relative_path`: lib/assets/fonts/Abel-Regular.ttf
`format`: TrueType
`size`: 33184   




### lib/assets/fonts/Poppins-Black.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/Poppins-Black.ttf
`relative_path`: lib/assets/fonts/Poppins-Black.ttf
`format`: TrueType
`size`: 151396   




### lib/assets/fonts/IBMPlexMono-SemiBold.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-SemiBold.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-SemiBold.ttf
`format`: TrueType
`size`: 138372   




### lib/assets/fonts/IBMPlexMono-SemiBoldItalic.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-SemiBoldItalic.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-SemiBoldItalic.ttf
`format`: TrueType
`size`: 145452   




### lib/assets/fonts/IBMPlexMono-LightItalic.ttf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/fonts/IBMPlexMono-LightItalic.ttf
`relative_path`: lib/assets/fonts/IBMPlexMono-LightItalic.ttf
`format`: TrueType
`size`: 141520   




### lib/assets/animations/unlockAnimation.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/animations/unlockAnimation.json
`relative_path`: lib/assets/animations/unlockAnimation.json
`format`: Arbitrary Binary Data
`size`: 11873   




### lib/assets/animations/waves.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/animations/waves.json
`relative_path`: lib/assets/animations/waves.json
`format`: Arbitrary Binary Data
`size`: 11790   




### lib/assets/animations/dots.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/animations/dots.json
`relative_path`: lib/assets/animations/dots.json
`format`: Arbitrary Binary Data
`size`: 2215030   




### lib/assets/animations/unlockAnimation.lottie
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/assets/animations/unlockAnimation.lottie
`relative_path`: lib/assets/animations/unlockAnimation.lottie
`format`: ZIP
`size`: 2749   




### lib/bottom_navigation_bar.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/bottom_navigation_bar.dart
`relative_path`: lib/bottom_navigation_bar.dart
`format`: Arbitrary Binary Data
`size`: 3823   


``````
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:Vacuul/screens/main/bookings.dart';
import 'package:Vacuul/screens/main/home.dart';
import 'package:Vacuul/screens/main/reading.dart';

class BottomNavigationBarWidget extends StatefulWidget {
  final int initialIndex; // Neuer Parameter für den initialen Index

  const BottomNavigationBarWidget({super.key, this.initialIndex = 0});

  @override
  State<BottomNavigationBarWidget> createState() =>
      _BottomNavigationBarWidgetState();
}

class _BottomNavigationBarWidgetState extends State<BottomNavigationBarWidget> {
  late int selectedIndex;

  @override
  void initState() {
    super.initState();
    selectedIndex = widget.initialIndex; // Initialisiere mit dem übergebenen Index
  }

  void _onItemTapped(int index) {
    setState(() {
      selectedIndex = index;
    });

    // Navigation basierend auf dem Tab-Index
    switch (index) {
      case 0:
        Navigator.pushReplacement(
          context,
          PageRouteBuilder(
            pageBuilder: (context, animation1, animation2) => const HomeScreen(),
            transitionDuration: Duration.zero,
            reverseTransitionDuration: Duration.zero,
          ),
        );
        break;
      case 1:
        Navigator.pushReplacement(
          context,
          PageRouteBuilder(
            pageBuilder: (context, animation1, animation2) => const Bookings(),
            transitionDuration: Duration.zero,
            reverseTransitionDuration: Duration.zero,
          ),
        );
        break;
      case 2:
        Navigator.pushReplacement(
          context,
          PageRouteBuilder(
            pageBuilder: (context, animation1, animation2) => const Reading(),
            transitionDuration: Duration.zero,
            reverseTransitionDuration: Duration.zero,
          ),
        );
        break;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.only(top: 8.0),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          _buildNavItem(0, "HOME", "lib/assets/svg/home.svg",
              "lib/assets/svg/homeunselected.svg"),
          _buildNavItem(1, "BOOKINGS", "lib/assets/svg/bookings.svg",
              "lib/assets/svg/bookingsunselected.svg"),
          _buildNavItem(2, "READING", "lib/assets/svg/reading.svg",
              "lib/assets/svg/readingunselected.svg"),
        ],
      ),
    );
  }

  Widget _buildNavItem(
      int index, String title, String selectedSvg, String unselectedSvg) {
    bool isSelected = selectedIndex == index;

    return TextButton(
      onPressed: () => _onItemTapped(index),
      style: TextButton.styleFrom(
        padding: EdgeInsets.zero, // Kein zusätzliches Padding
        minimumSize: const Size(48, 48), // Klickbarer Bereich, ohne die Größe zu verändern
        tapTargetSize: MaterialTapTargetSize.shrinkWrap, // Minimaler Touch-Bereich
        backgroundColor: Colors.transparent, // Transparenter Hintergrund
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          SvgPicture.asset(
            isSelected ? selectedSvg : unselectedSvg,
            height: 24.0,
            width: 24.0,
          ),
          const SizedBox(height: 4.0), // Abstand zwischen Icon und Text
          Text(
            title,
            style: TextStyle(
              fontFamily: 'IBM Plex Mono',
              fontSize: 10.0,
              fontWeight: isSelected ? FontWeight.w600 : FontWeight.w400,
              color: isSelected
                  ? const Color.fromRGBO(28, 27, 31, 1)
                  : const Color.fromRGBO(81, 73, 75, 1),
            ),
          ),
        ],
      ),
    );
  }
}

``````


### lib/widgets/news_slider.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/widgets/news_slider.dart
`relative_path`: lib/widgets/news_slider.dart
`format`: Arbitrary Binary Data
`size`: 2389   


``````
import 'package:flutter/material.dart';
import 'package:Vacuul/screens/main/reaading_detail.dart';

class NewsSlider extends StatelessWidget {
  const NewsSlider({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.only(top: 8.0, left: 0.0, right: 0.0, bottom: 12.0),
      child: SizedBox(
        height: 200.0, // Gesamthöhe des Sliders
        child: ListView(
          scrollDirection: Axis.horizontal,
          children: [
            _buildSliderItem(
              imagePath: 'lib/assets/images/news1.png',
              title: 'What is Red Light Therapy?',
              index: 0,
              context: context
            ),
            _buildSliderItem(
              imagePath: 'lib/assets/images/news2.png',
              title: 'Daily Usage & Health',
              index: 1,
              context: context
            ),
            _buildSliderItem(
              imagePath: 'lib/assets/images/news3.png',
              title: 'All about waves',
              index: 2,
              context: context
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSliderItem({required String imagePath, required String title, required int index, required BuildContext context}) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 8.0),
      child: GestureDetector(
      onTap: () => {
        Navigator.of(context).push(PageRouteBuilder(
          pageBuilder: (context, animation, secondaryAnimation) => ReadingDetails(detailIndex: index),
        ))
      },
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          // Image
          Container(
            width: 160.0,
            height: 128.0,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(8.0),
              image: DecorationImage(
                image: AssetImage(imagePath),
                fit: BoxFit.cover,
              ),
            ),
          ),
          const SizedBox(height: 8.0),
          // Text
          Text(
            title,
            style: const TextStyle(
              fontFamily: 'Abel',
              fontSize: 16.0,
              color: Color.fromRGBO(28, 27, 31, 1),
            ),
            textAlign: TextAlign.center,
          ),
        ],
      ),
      ),
    );
  }
}

``````


### lib/widgets/slideToAct.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/widgets/slideToAct.dart
`relative_path`: lib/widgets/slideToAct.dart
`format`: Arbitrary Binary Data
`size`: 12790   




### lib/widgets/animated_donut_chart.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/widgets/animated_donut_chart.dart
`relative_path`: lib/widgets/animated_donut_chart.dart
`format`: Arbitrary Binary Data
`size`: 3610   


``````
import 'package:flutter/material.dart';

class AnimatedDonutChart extends StatefulWidget {
  const AnimatedDonutChart({super.key});

  @override
  State<AnimatedDonutChart> createState() => _AnimatedDonutChartState();
}

class _AnimatedDonutChartState extends State<AnimatedDonutChart> with TickerProviderStateMixin {
  late List<AnimationController> _controllers;
  late List<Animation<double>> _animations;
  final int _totalSegments = 8; // Anzahl der Segmente
  final Duration _segmentDelay = const Duration(milliseconds: 300); // Verzögerung zwischen Segmenten

  @override
  void initState() {
    super.initState();

    // Initialisierung der AnimationController
    _controllers = List.generate(
      _totalSegments,
          (index) => AnimationController(
        vsync: this,
        duration: const Duration(milliseconds: 500), // Dauer der Fade-In-Animation pro Segment
      ),
    );

    // Animationen (Tween von 0 bis 1) für jedes Segment
    _animations = _controllers.map((controller) {
      return Tween<double>(begin: 0.0, end: 1.0).animate(
        CurvedAnimation(parent: controller, curve: Curves.easeIn),
      );
    }).toList();

    // Starte die Animationen mit Verzögerung nacheinander
    for (int i = 0; i < _controllers.length; i++) {
      Future.delayed(_segmentDelay * i, () {
        if (mounted) {
          _controllers[i].forward();
        }
      });
    }
  }

  @override
  void dispose() {
    // Entsorge alle AnimationController
    for (var controller in _controllers) {
      controller.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Stack(
      alignment: Alignment.center,
      children: [
        SizedBox(
          width: 96,
          height: 96,
          child: CustomPaint(
            painter: AnimatedDonutChartPainter(
              animations: _animations,
              totalSegments: _totalSegments,
            ),
          ),
        ),
        // Text in der Mitte
        const Center(
          child: Text(
            '0%',
            style: TextStyle(
              color: Color.fromRGBO(28, 27, 31, 1),
              fontFamily: 'IBM Plex Mono',
              fontWeight: FontWeight.w600,
              fontSize: 18.0,
            ),
          ),
        ),
      ],
    );
  }
}

class AnimatedDonutChartPainter extends CustomPainter {
  final List<Animation<double>> animations;
  final int totalSegments;

  AnimatedDonutChartPainter({
    required this.animations,
    required this.totalSegments,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final strokeWidth = size.width * 0.1;
    final gapAngle = 15.0;
    final sweepAngle = (360 - (gapAngle * totalSegments)) / totalSegments;

    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = strokeWidth
      ..strokeCap = StrokeCap.round;

    final rect = Rect.fromLTWH(
      strokeWidth / 2,
      strokeWidth / 2,
      size.width - strokeWidth,
      size.height - strokeWidth,
    );

    for (int i = 0; i < totalSegments; i++) {
      final startAngle = -90 + i * (sweepAngle + gapAngle);
      final opacity = animations[i].value;

      paint.color = const Color.fromRGBO(219, 220, 219, 1).withOpacity(opacity);

      canvas.drawArc(
        rect,
        _degreesToRadians(startAngle),
        _degreesToRadians(sweepAngle),
        false,
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => true;

  double _degreesToRadians(double degrees) {
    return degrees * (3.141592653589793 / 180);
  }
}

``````


### lib/widgets/spinnerPainter.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/widgets/spinnerPainter.dart
`relative_path`: lib/widgets/spinnerPainter.dart
`format`: Arbitrary Binary Data
`size`: 844   


``````
import 'package:flutter/material.dart';

class SpinnerPainter extends CustomPainter {
  final double rotationValue;

  SpinnerPainter(this.rotationValue);

  @override
  void paint(Canvas canvas, Size size) {
    final Paint paint = Paint()
      ..color = Colors.grey
      ..strokeWidth = 4.0
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;

    final double startAngle = 2 * 3.14159 * rotationValue; // Startwinkel basierend auf der Rotation
    final double sweepAngle = 3.14159 / 3; // Länge des sichtbaren Segments (z. B. 60°)

    final Rect rect = Rect.fromCircle(
      center: Offset(size.width / 2, size.height / 2),
      radius: size.width / 2,
    );

    canvas.drawArc(rect, startAngle, sweepAngle, false, paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => true;
}

``````


### lib/widgets/top_bar.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/widgets/top_bar.dart
`relative_path`: lib/widgets/top_bar.dart
`format`: Arbitrary Binary Data
`size`: 3165   


``````
import 'package:flutter/material.dart';

class TopBar extends StatelessWidget {
  final String title;
  final bool backButton;
  final bool cancelButton;

  const TopBar({
    super.key,
    required this.title,
    this.backButton = false,
    this.cancelButton = false
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 12.0),
      decoration: const BoxDecoration(
        border: Border(
          bottom: BorderSide(
            color: Color(0xFF9BA19B),
            width: 1.0,
          ),
        ),
      ),
      child: backButton
          ? Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          SizedBox(
            width: 24,
            height: 24,
            child: TextButton(
              onPressed: () {
                Navigator.pop(context);
              },
              style: TextButton.styleFrom(
                padding: EdgeInsets.zero,
                minimumSize: Size.zero,
                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
              ),
              child: const Icon(
                Icons.arrow_back_ios_new,
                size: 24,
                color: Color.fromRGBO(28, 27, 31, 1),
              ),
            ),
          ),
          // Titel
          Text(
            title,
            style: const TextStyle(
              fontWeight: FontWeight.w500,
              fontFamily: 'IBM Plex Mono',
              fontSize: 16.0,
              color: Color.fromRGBO(28, 27, 31, 1),
            ),
          ),
          // Platzhalter
          const SizedBox(width: 24),
        ],
      )
          : cancelButton ?
      Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          SizedBox(
            width: 24,
            height: 24,
            child: TextButton(
              onPressed: () {
                Navigator.of(context).pushNamedAndRemoveUntil(
                  "/home",
                      (route) => false,
                );
              },
              style: TextButton.styleFrom(
                padding: EdgeInsets.zero,
                minimumSize: Size.zero,
                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
              ),
              child: const Icon(
                Icons.close,
                size: 24,
                color: Color.fromRGBO(28, 27, 31, 1),
              ),
            ),
          ),
          // Titel
          Text(
            title,
            style: const TextStyle(
              fontWeight: FontWeight.w500,
              fontFamily: 'IBM Plex Mono',
              fontSize: 16.0,
              color: Color.fromRGBO(28, 27, 31, 1),
            ),
          ),
          // Platzhalter
          const SizedBox(width: 24),
        ],
      )
          : Center(
        child: Text(
          title,
          style: const TextStyle(
            fontWeight: FontWeight.w500,
            fontFamily: 'IBM Plex Mono',
            fontSize: 16.0,
            color: Color.fromRGBO(28, 27, 31, 1),
          ),
        ),
      ),
    );
  }
}

``````


### lib/widgets/donut_chart.dart
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/lib/widgets/donut_chart.dart
`relative_path`: lib/widgets/donut_chart.dart
`format`: Arbitrary Binary Data
`size`: 1486   


``````
import 'package:flutter/material.dart';

class DonutChartPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final strokeWidth = size.width * 0.1; // Dynamische Strichbreite (10% der Größe)
    final gapAngle = 15.0; // Lücke zwischen den Segmenten in Grad
    final totalSegments = 8; // Anzahl der Segmente
    final sweepAngle = (360 - (gapAngle * totalSegments)) / totalSegments; // Segmentwinkel

    final paint = Paint()
      ..color = const Color.fromRGBO(219, 220, 219, 1) // Farbe der Segmente
      ..style = PaintingStyle.stroke
      ..strokeWidth = strokeWidth
      ..strokeCap = StrokeCap.round; // Runde Enden für saubere Abstände

    // Rechteck für den Donut
    final rect = Rect.fromLTWH(
      strokeWidth / 2, // Berücksichtigt die Strichbreite, um innerhalb der Fläche zu bleiben
      strokeWidth / 2,
      size.width - strokeWidth,
      size.height - strokeWidth,
    );

    // Zeichne die Segmente
    for (int i = 0; i < totalSegments; i++) {
      final startAngle = -90 + i * (sweepAngle + gapAngle); // Startwinkel
      canvas.drawArc(
        rect,
        _degreesToRadians(startAngle),
        _degreesToRadians(sweepAngle),
        false,
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;

  // Hilfsmethode: Grad in Bogenmaß umrechnen
  double _degreesToRadians(double degrees) {
    return degrees * (3.141592653589793 / 180);
  }
}
``````


### analysis_options.yaml
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/app/analysis_options.yaml
`relative_path`: analysis_options.yaml
`format`: Arbitrary Binary Data
`size`: 1420   


``````
# This file configures the analyzer, which statically analyzes Dart code to
# check for errors, warnings, and lints.
#
# The issues identified by the analyzer are surfaced in the UI of Dart-enabled
# IDEs (https://dart.dev/tools#ides-and-editors). The analyzer can also be
# invoked from the command line by running `flutter analyze`.

# The following line activates a set of recommended lints for Flutter apps,
# packages, and plugins designed to encourage good coding practices.
include: package:flutter_lints/flutter.yaml

linter:
  # The lint rules applied to this project can be customized in the
  # section below to disable rules from the `package:flutter_lints/flutter.yaml`
  # included above or to enable additional rules. A list of all available lints
  # and their documentation is published at https://dart.dev/lints.
  #
  # Instead of disabling a lint rule for the entire project in the
  # section below, it can also be suppressed for a single line of code
  # or a specific dart file by using the `// ignore: name_of_lint` and
  # `// ignore_for_file: name_of_lint` syntax on the line or in the file
  # producing the lint.
  rules:
    # avoid_print: false  # Uncomment to disable the `avoid_print` rule
    # prefer_single_quotes: true  # Uncomment to enable the `prefer_single_quotes` rule

# Additional information about this file can be found at
# https://dart.dev/guides/language/analysis-options

``````



## api-specification
`clone_url`: https://github.com/vacuul-dev/api-specification.git


### diagrams.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/diagrams.md
`relative_path`: diagrams.md
`format`: Arbitrary Binary Data
`size`: 1048   


``````
# Diagrams

## System Overview
```mermaid
graph TB
    BACKEND[(Backend)] <--(I)--> MOBILE([User Mobile App])
    BACKEND <--(II) HTTP JSON API--> RPI[Vacuul Machine]
    RPI <--(III) WebSocket--> DISP[Display WebApp]
    DISP <--(IV)--> BACKEND
```

## Interface II
HTTP GET/PUSH JSON API

```mermaid
graph TB
    S((Start)) --> COMIS{{Machine ID is set?}}
    COMIS --n--> APIcomisInterval(poll interval: 3..8s)
    APIcomisInterval --> APIcomis(HTTP POST /api/machines)
    APIcomis --> COMIS
    COMIS --y--> APIstart(HTTP POST /api/machines/start)
    APIstart --> START{{Start treatment?}}
    START --n--> APIstartInterval(poll interval: 2..7s)
    APIstartInterval --> APIstart
    START --y--> APIsettings(HTTP POST /api/machines/settings)
    APIsettings --> APItreat(HTTP POST /api/machines/progress)
    APItreat --> DONE{{Treatment done or aborted?}}
    DONE --n--> APIprogressInterval(poll interval: 2..7s)
    APIprogressInterval --> APItreat
    DONE --y--> FINAL(final HTTP POST /api/machines/progress)
    FINAL --> APIstart
```

``````


### req-bodies/error-req-v1.1.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/req-bodies/error-req-v1.1.json
`relative_path`: req-bodies/error-req-v1.1.json
`format`: Arbitrary Binary Data
`size`: 2822   


``````
{
    "appData": {
        "appState": "error"
    },
    "machine_id": "asdf1234",
    "status": {
        "airTemp": 23.241151809692383,
        "connection": {
            "backend": true,
            "internet": true
        },
        "debugInfo": {
            "max32664": {
                "info": {
                    "afeType": "MAX3010x",
                    "afeVersion": "6",
                    "chipVersion": "A",
                    "version": "10.3.0"
                }
            },
            "messages": [
                {
                    "msg": "I2C read mcp9808 failed",
                    "t": 1738664231
                },
                {
                    "msg": "I2C read mcp9808 failed",
                    "t": 1738664232
                },
                {
                    "msg": "I2C read mcp9808 failed",
                    "t": 1738664232
                },
                {
                    "msg": "I2C read mcp9808 failed",
                    "t": 1738664232
                },
                {
                    "msg": "I2C read mcp9808 failed",
                    "t": 1738664232
                },
                {
                    "msg": "I2C read mcp9808 failed",
                    "t": 1738664232
                },
                {
                    "msg": "I2C read mcp9808 failed",
                    "t": 1738664233
                },
                {
                    "msg": "I2C read mcp9808 failed",
                    "t": 1738664233
                },
                {
                    "msg": "I2C read mcp9808 failed",
                    "t": 1738664233
                },
                {
                    "msg": "I2C read mcp9808 failed",
                    "t": 1738664233
                },
                {
                    "msg": "[I2C Thread] restart",
                    "t": 1738664233
                },
                {
                    "msg": "failed to connect to mcp9808",
                    "t": 1738664233
                },
                {
                    "msg": "failed to connect to mcp9808",
                    "t": 1738664234
                },
                {
                    "msg": "failed to connect to mcp9808",
                    "t": 1738664235
                },
                {
                    "msg": "failed to connect to mcp9808",
                    "t": 1738664236
                },
                {
                    "msg": "[I2C Thread] failed to establish connection to all I2C devices",
                    "t": 1738664236
                }
            ],
            "peripheralErrorFlags": 20,
            "threadBootFlags": 0
        },
        "deviceInfo": {
            "swVersion": "0.0.0"
        },
        "machine_id": "asdf1234"
    }
}
``````


### req-bodies/progress-req-v1.1.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/req-bodies/progress-req-v1.1.json
`relative_path`: req-bodies/progress-req-v1.1.json
`format`: Arbitrary Binary Data
`size`: 1497   


``````
{
    "appData": {
        "abortReason": 0,
        "appState": "treatment",
        "biometrics": {
            "algorithmState": 0,
            "algorithmStatus": -3,
            "heartRate": 0.0,
            "oxygenSaturation": 0.0
        },
        "blockType": "treat",
        "elapsedTime": 0,
        "handsOn": true,
        "noHandsTimeout": 1,
        "peltier": {
            "left": {
                "setPoint": 17.0,
                "temperature": -11.386545181274414,
                "tolerance": 0.8999999761581421
            },
            "right": {
                "setPoint": 17.0,
                "temperature": 15.814253807067871,
                "tolerance": 0.8999999761581421
            }
        },
        "treatmentDuration": 30,
        "treatmentTime": 0,
        "userName": "User#4099"
    },
    "machine_id": "asdf1234",
    "status": {
        "airTemp": 23.274410247802734,
        "connection": {
            "backend": true,
            "internet": true
        },
        "debugInfo": {
            "max32664": {
                "info": {
                    "afeType": "MAX3010x",
                    "afeVersion": "6",
                    "chipVersion": "A",
                    "version": "10.3.0"
                }
            },
            "messages": [],
            "peripheralErrorFlags": 0,
            "threadBootFlags": 0
        },
        "deviceInfo": {
            "swVersion": "0.0.0"
        },
        "machine_id": "asdf1234"
    }
}
``````


### req-bodies/start-req-v1.1.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/req-bodies/start-req-v1.1.json
`relative_path`: req-bodies/start-req-v1.1.json
`format`: Arbitrary Binary Data
`size`: 724   


``````
{
    "appData": {
        "appState": "idle"
    },
    "machine_id": "asdf1234",
    "status": {
        "airTemp": 22.51666831970215,
        "connection": {
            "backend": false,
            "internet": true
        },
        "debugInfo": {
            "max32664": {
                "info": {
                    "afeType": "unknown",
                    "afeVersion": "",
                    "chipVersion": "unknown",
                    "version": "0.0.0"
                }
            },
            "messages": [],
            "peripheralErrorFlags": 16,
            "threadBootFlags": 0
        },
        "deviceInfo": {
            "swVersion": "0.0.0"
        },
        "machine_id": "asdf1234"
    }
}
``````


### req-bodies/commission-req-v1.2.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/req-bodies/commission-req-v1.2.json
`relative_path`: req-bodies/commission-req-v1.2.json
`format`: Arbitrary Binary Data
`size`: 707   


``````
{
    "commissionId": "vabv-kbNt-W73N-6jKJ",
    "status": {
        "airTemp": 22.53172492980957,
        "connection": {
            "backend": false,
            "internet": true
        },
        "debugInfo": {
            "max32664": {
                "info": {
                    "afeType": "MAX3010x",
                    "afeVersion": "6",
                    "chipVersion": "A",
                    "version": "10.3.0"
                }
            },
            "messages": [],
            "peripheralErrorFlags": 0,
            "threadBootFlags": 0
        },
        "deviceInfo": {
            "hwVersion": "2.2",
            "swVersion": "0.0.0"
        },
        "machine_id": "0"
    }
}
``````


### req-bodies/settings-req-v1.2.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/req-bodies/settings-req-v1.2.json
`relative_path`: req-bodies/settings-req-v1.2.json
`format`: Arbitrary Binary Data
`size`: 752   


``````
{
    "appData": {
        "appState": "idle"
    },
    "machine_id": "asdf1234",
    "status": {
        "airTemp": 22.758150100708008,
        "connection": {
            "backend": true,
            "internet": true
        },
        "debugInfo": {
            "max32664": {
                "info": {
                    "afeType": "MAX3010x",
                    "afeVersion": "6",
                    "chipVersion": "A",
                    "version": "10.3.0"
                }
            },
            "messages": [],
            "peripheralErrorFlags": 0,
            "threadBootFlags": 0
        },
        "deviceInfo": {
            "hwVersion": "2.2",
            "swVersion": "0.0.0"
        },
        "machine_id": "asdf1234"
    }
}
``````


### req-bodies/settings-req-v1.1.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/req-bodies/settings-req-v1.1.json
`relative_path`: req-bodies/settings-req-v1.1.json
`format`: Arbitrary Binary Data
`size`: 720   


``````
{
    "appData": {
        "appState": "idle"
    },
    "machine_id": "asdf1234",
    "status": {
        "airTemp": 23.276487350463867,
        "connection": {
            "backend": true,
            "internet": true
        },
        "debugInfo": {
            "max32664": {
                "info": {
                    "afeType": "MAX3010x",
                    "afeVersion": "6",
                    "chipVersion": "A",
                    "version": "10.3.0"
                }
            },
            "messages": [],
            "peripheralErrorFlags": 0,
            "threadBootFlags": 0
        },
        "deviceInfo": {
            "swVersion": "0.0.0"
        },
        "machine_id": "asdf1234"
    }
}
``````


### req-bodies/progress-req-v1.2.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/req-bodies/progress-req-v1.2.json
`relative_path`: req-bodies/progress-req-v1.2.json
`format`: Arbitrary Binary Data
`size`: 1527   


``````
{
    "appData": {
        "abortReason": 0,
        "appState": "treatment",
        "biometrics": {
            "algorithmState": 0,
            "algorithmStatus": -3,
            "heartRate": 0.0,
            "oxygenSaturation": 0.0
        },
        "blockType": "treat",
        "elapsedTime": 2,
        "handsOn": true,
        "noHandsTimeout": 5,
        "peltier": {
            "left": {
                "setPoint": 14.050000190734863,
                "temperature": -11.386545181274414,
                "tolerance": 0.5
            },
            "right": {
                "setPoint": 14.050000190734863,
                "temperature": 12.538225173950195,
                "tolerance": 0.5
            }
        },
        "treatmentDuration": 30,
        "treatmentTime": 2,
        "userName": "User#4511"
    },
    "machine_id": "asdf1234",
    "status": {
        "airTemp": 22.754785537719727,
        "connection": {
            "backend": true,
            "internet": true
        },
        "debugInfo": {
            "max32664": {
                "info": {
                    "afeType": "MAX3010x",
                    "afeVersion": "6",
                    "chipVersion": "A",
                    "version": "10.3.0"
                }
            },
            "messages": [],
            "peripheralErrorFlags": 0,
            "threadBootFlags": 0
        },
        "deviceInfo": {
            "hwVersion": "2.2",
            "swVersion": "0.0.0"
        },
        "machine_id": "asdf1234"
    }
}
``````


### req-bodies/start-req-v1.2.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/req-bodies/start-req-v1.2.json
`relative_path`: req-bodies/start-req-v1.2.json
`format`: Arbitrary Binary Data
`size`: 756   


``````
{
    "appData": {
        "appState": "idle"
    },
    "machine_id": "asdf1234",
    "status": {
        "airTemp": 22.624797821044922,
        "connection": {
            "backend": true,
            "internet": true
        },
        "debugInfo": {
            "max32664": {
                "info": {
                    "afeType": "unknown",
                    "afeVersion": "",
                    "chipVersion": "unknown",
                    "version": "0.0.0"
                }
            },
            "messages": [],
            "peripheralErrorFlags": 16,
            "threadBootFlags": 0
        },
        "deviceInfo": {
            "hwVersion": "2.2",
            "swVersion": "0.0.0"
        },
        "machine_id": "asdf1234"
    }
}
``````


### req-bodies/commission-req-v1.1.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/req-bodies/commission-req-v1.1.json
`relative_path`: req-bodies/commission-req-v1.1.json
`format`: Arbitrary Binary Data
`size`: 676   


``````
{
    "commissionId": "xyoM-paPh-7str-byD6",
    "status": {
        "airTemp": 22.584503173828125,
        "connection": {
            "backend": false,
            "internet": true
        },
        "debugInfo": {
            "max32664": {
                "info": {
                    "afeType": "MAX3010x",
                    "afeVersion": "6",
                    "chipVersion": "A",
                    "version": "10.3.0"
                }
            },
            "messages": [],
            "peripheralErrorFlags": 0,
            "threadBootFlags": 0
        },
        "deviceInfo": {
            "swVersion": "0.0.0"
        },
        "machine_id": "0"
    }
}
``````


### req-bodies/error-req-v1.2.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/req-bodies/error-req-v1.2.json
`relative_path`: req-bodies/error-req-v1.2.json
`format`: Arbitrary Binary Data
`size`: 1564   


``````
{
    "appData": {
        "appState": "error"
    },
    "machine_id": "asdf1234",
    "status": {
        "airTemp": 0.0,
        "connection": {
            "backend": true,
            "internet": true
        },
        "debugInfo": {
            "max32664": {
                "info": {
                    "afeType": "unknown",
                    "afeVersion": "",
                    "chipVersion": "unknown",
                    "version": "0.0.0"
                }
            },
            "messages": [
                {
                    "msg": "failed to connect to mcp9808",
                    "t": 1738664341
                },
                {
                    "msg": "failed to connect to mcp9808",
                    "t": 1738664342
                },
                {
                    "msg": "failed to connect to mcp9808",
                    "t": 1738664343
                },
                {
                    "msg": "failed to connect to mcp9808",
                    "t": 1738664344
                },
                {
                    "msg": "[I2C Thread] failed to establish connection to all I2C devices",
                    "t": 1738664344
                },
                {
                    "msg": "boot timeout",
                    "t": 1738664401
                }
            ],
            "peripheralErrorFlags": 20,
            "threadBootFlags": 2
        },
        "deviceInfo": {
            "hwVersion": "2.2",
            "swVersion": "0.0.0"
        },
        "machine_id": "asdf1234"
    }
}
``````


### API-Endpoints_v1.2_250123.pdf
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/API-Endpoints_v1.2_250123.pdf
`relative_path`: API-Endpoints_v1.2_250123.pdf
`format`: Portable Document Format
`size`: 330917   




### readme.md
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/readme.md
`relative_path`: readme.md
`format`: Arbitrary Binary Data
`size`: 843   


``````
# API Specification

[API Endpoints](./API-Endpoints_v1.2_250123.pdf)



## JSON Schemas for Validating JSON Bodies of API

[JSON schemas](./json-schema/)

Validator: [jsonschemavalidator.net](https://www.jsonschemavalidator.net/).



------------------------------------------------------------------------------------------------------------------------

## Testing Machine Firmware

### Request
#### API Version _on Machine_ vs _on API_
| &#x2193; Machine \ API &#x2192; |   v1.1   |   v1.2   |
|:-------------------------------:|:--------:|:--------:|
| v1.1                            | &#x2705; | - [^1]   |
| v1.2                            | &#x2705; | &#x2705; |

[^1]: If the `"hwVersion"` is not marked as _required_ it would also be OK. But the machine firmware (>`v0.3.0-alpha`) has v1.2 implemented anyway, so this can't happen.

``````


### json-schema/commission-req.schema.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/json-schema/commission-req.schema.json
`relative_path`: json-schema/commission-req.schema.json
`format`: Arbitrary Binary Data
`size`: 4671   


``````
{
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://example.com/api-v1.2/commission-req.schema.json",
    "type": "object",
    "properties": {
        "commissionId": {
            "type": "string"
        },
        "status": {
            "type": "object",
            "properties": {
                "airTemp": {
                    "type": "number"
                },
                "connection": {
                    "type": "object",
                    "properties": {
                        "backend": {
                            "type": "boolean"
                        },
                        "internet": {
                            "type": "boolean"
                        }
                    },
                    "required": [
                        "backend",
                        "internet"
                    ]
                },
                "debugInfo": {
                    "type": "object",
                    "properties": {
                        "peripheralErrorFlags": {
                            "type": "integer"
                        },
                        "threadBootFlags": {
                            "type": "integer"
                        },
                        "messages": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "t": {
                                        "type": "integer"
                                    },
                                    "msg": {
                                        "type": "string"
                                    }
                                },
                                "required": [
                                    "t",
                                    "msg"
                                ]
                            },
                            "minItems": 0,
                            "uniqueItems": false
                        },
                        "max32664": {
                            "type": "object",
                            "properties": {
                                "info": {
                                    "type": "object",
                                    "properties": {
                                        "afeType": {
                                            "type": "string"
                                        },
                                        "afeVersion": {
                                            "type": "string"
                                        },
                                        "chipVersion": {
                                            "type": "string"
                                        },
                                        "version": {
                                            "type": "string"
                                        }
                                    },
                                    "required": [
                                        "afeType",
                                        "afeVersion",
                                        "chipVersion",
                                        "version"
                                    ]
                                }
                            },
                            "required": [
                                "info"
                            ]
                        }
                    },
                    "required": [
                        "peripheralErrorFlags",
                        "threadBootFlags",
                        "messages",
                        "max32664"
                    ]
                },
                "deviceInfo": {
                    "type": "object",
                    "properties": {
                        "hwVersion": {
                            "type": "string"
                        },
                        "swVersion": {
                            "type": "string"
                        }
                    },
                    "required": [
                        "hwVersion",
                        "swVersion"
                    ]
                },
                "machine_id": {
                    "type": "string"
                }
            },
            "required": [
                "airTemp",
                "connection",
                "debugInfo",
                "deviceInfo",
                "machine_id"
            ]
        }
    },
    "required": [
        "commissionId",
        "status"
    ]
}
``````


### json-schema/progress-req.schema.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/json-schema/progress-req.schema.json
`relative_path`: json-schema/progress-req.schema.json
`format`: Arbitrary Binary Data
`size`: 5911   




### json-schema/settings-req.schema.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/json-schema/settings-req.schema.json
`relative_path`: json-schema/settings-req.schema.json
`format`: Arbitrary Binary Data
`size`: 4665   


``````
{
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://example.com/api-v1.2/settings-req.schema.json",
    "type": "object",
    "properties": {
        "machine_id": {
            "type": "string"
        },
        "status": {
            "type": "object",
            "properties": {
                "airTemp": {
                    "type": "number"
                },
                "connection": {
                    "type": "object",
                    "properties": {
                        "backend": {
                            "type": "boolean"
                        },
                        "internet": {
                            "type": "boolean"
                        }
                    },
                    "required": [
                        "backend",
                        "internet"
                    ]
                },
                "debugInfo": {
                    "type": "object",
                    "properties": {
                        "peripheralErrorFlags": {
                            "type": "integer"
                        },
                        "threadBootFlags": {
                            "type": "integer"
                        },
                        "messages": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "t": {
                                        "type": "integer"
                                    },
                                    "msg": {
                                        "type": "string"
                                    }
                                },
                                "required": [
                                    "t",
                                    "msg"
                                ]
                            },
                            "minItems": 0,
                            "uniqueItems": false
                        },
                        "max32664": {
                            "type": "object",
                            "properties": {
                                "info": {
                                    "type": "object",
                                    "properties": {
                                        "afeType": {
                                            "type": "string"
                                        },
                                        "afeVersion": {
                                            "type": "string"
                                        },
                                        "chipVersion": {
                                            "type": "string"
                                        },
                                        "version": {
                                            "type": "string"
                                        }
                                    },
                                    "required": [
                                        "afeType",
                                        "afeVersion",
                                        "chipVersion",
                                        "version"
                                    ]
                                }
                            },
                            "required": [
                                "info"
                            ]
                        }
                    },
                    "required": [
                        "peripheralErrorFlags",
                        "threadBootFlags",
                        "messages",
                        "max32664"
                    ]
                },
                "deviceInfo": {
                    "type": "object",
                    "properties": {
                        "hwVersion": {
                            "type": "string"
                        },
                        "swVersion": {
                            "type": "string"
                        }
                    },
                    "required": [
                        "hwVersion",
                        "swVersion"
                    ]
                },
                "machine_id": {
                    "type": "string"
                }
            },
            "required": [
                "airTemp",
                "connection",
                "debugInfo",
                "deviceInfo",
                "machine_id"
            ]
        }
    },
    "required": [
        "machine_id",
        "status"
    ]
}
``````


### json-schema/error-req.schema.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/json-schema/error-req.schema.json
`relative_path`: json-schema/error-req.schema.json
`format`: Arbitrary Binary Data
`size`: 4938   


``````
{
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://example.com/api-v1.2/error-req.schema.json",
    "type": "object",
    "properties": {
        "machine_id": {
            "type": "string"
        },
        "appData": {
            "type": "object",
            "properties": {
                "appState": {
                    "type": "string"
                }
            },
            "required": [
                "appState"
            ]
        },
        "status": {
            "type": "object",
            "properties": {
                "airTemp": {
                    "type": "number"
                },
                "connection": {
                    "type": "object",
                    "properties": {
                        "backend": {
                            "type": "boolean"
                        },
                        "internet": {
                            "type": "boolean"
                        }
                    },
                    "required": [
                        "backend",
                        "internet"
                    ]
                },
                "debugInfo": {
                    "type": "object",
                    "properties": {
                        "peripheralErrorFlags": {
                            "type": "integer"
                        },
                        "threadBootFlags": {
                            "type": "integer"
                        },
                        "messages": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "t": {
                                        "type": "integer"
                                    },
                                    "msg": {
                                        "type": "string"
                                    }
                                },
                                "required": [
                                    "t",
                                    "msg"
                                ]
                            },
                            "minItems": 0,
                            "uniqueItems": false
                        },
                        "max32664": {
                            "type": "object",
                            "properties": {
                                "info": {
                                    "type": "object",
                                    "properties": {
                                        "afeType": {
                                            "type": "string"
                                        },
                                        "afeVersion": {
                                            "type": "string"
                                        },
                                        "chipVersion": {
                                            "type": "string"
                                        },
                                        "version": {
                                            "type": "string"
                                        }
                                    },
                                    "required": [
                                        "afeType",
                                        "afeVersion",
                                        "chipVersion",
                                        "version"
                                    ]
                                }
                            },
                            "required": [
                                "info"
                            ]
                        }
                    },
                    "required": [
                        "peripheralErrorFlags",
                        "threadBootFlags",
                        "messages",
                        "max32664"
                    ]
                },
                "deviceInfo": {
                    "type": "object",
                    "properties": {
                        "hwVersion": {
                            "type": "string"
                        },
                        "swVersion": {
                            "type": "string"
                        }
                    },
                    "required": [
                        "hwVersion",
                        "swVersion"
                    ]
                },
                "machine_id": {
                    "type": "string"
                }
            },
            "required": [
                "airTemp",
                "connection",
                "debugInfo",
                "deviceInfo",
                "machine_id"
            ]
        }
    },
    "required": [
        "machine_id",
        "appData",
        "status"
    ]
}
``````


### json-schema/start-req.schema.json
`path`: /var/folders/k2/gxrql7_14cz20q6kpq4jxspw0000gn/T/api-specification/json-schema/start-req.schema.json
`relative_path`: json-schema/start-req.schema.json
`format`: Arbitrary Binary Data
`size`: 4662   


``````
{
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://example.com/api-v1.2/start-req.schema.json",
    "type": "object",
    "properties": {
        "machine_id": {
            "type": "string"
        },
        "status": {
            "type": "object",
            "properties": {
                "airTemp": {
                    "type": "number"
                },
                "connection": {
                    "type": "object",
                    "properties": {
                        "backend": {
                            "type": "boolean"
                        },
                        "internet": {
                            "type": "boolean"
                        }
                    },
                    "required": [
                        "backend",
                        "internet"
                    ]
                },
                "debugInfo": {
                    "type": "object",
                    "properties": {
                        "peripheralErrorFlags": {
                            "type": "integer"
                        },
                        "threadBootFlags": {
                            "type": "integer"
                        },
                        "messages": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "t": {
                                        "type": "integer"
                                    },
                                    "msg": {
                                        "type": "string"
                                    }
                                },
                                "required": [
                                    "t",
                                    "msg"
                                ]
                            },
                            "minItems": 0,
                            "uniqueItems": false
                        },
                        "max32664": {
                            "type": "object",
                            "properties": {
                                "info": {
                                    "type": "object",
                                    "properties": {
                                        "afeType": {
                                            "type": "string"
                                        },
                                        "afeVersion": {
                                            "type": "string"
                                        },
                                        "chipVersion": {
                                            "type": "string"
                                        },
                                        "version": {
                                            "type": "string"
                                        }
                                    },
                                    "required": [
                                        "afeType",
                                        "afeVersion",
                                        "chipVersion",
                                        "version"
                                    ]
                                }
                            },
                            "required": [
                                "info"
                            ]
                        }
                    },
                    "required": [
                        "peripheralErrorFlags",
                        "threadBootFlags",
                        "messages",
                        "max32664"
                    ]
                },
                "deviceInfo": {
                    "type": "object",
                    "properties": {
                        "hwVersion": {
                            "type": "string"
                        },
                        "swVersion": {
                            "type": "string"
                        }
                    },
                    "required": [
                        "hwVersion",
                        "swVersion"
                    ]
                },
                "machine_id": {
                    "type": "string"
                }
            },
            "required": [
                "airTemp",
                "connection",
                "debugInfo",
                "deviceInfo",
                "machine_id"
            ]
        }
    },
    "required": [
        "machine_id",
        "status"
    ]
}
``````



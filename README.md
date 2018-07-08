# barbershop [![Build Status](https://travis-ci.org/meetup/barbershop.svg?branch=master)](https://travis-ci.org/meetup/barbershop) [![Coverage Status](https://coveralls.io/repos/github/meetup/barbershop/badge.svg?branch=master)](https://coveralls.io/github/meetup/barbershop?branch=master)

> trims a little of the top and sides of your github repository after pull request merges

## 🤔 about

This application is a github webhook handler for pull request events that automates the process of
deleting branches after a pull request is merged, keeping your github repository branch list
well trimmed 💇🏽‍♀️ 💇🏿‍♂️

## 🔌 install

You can install this application as a webook under your github repository's settings.

Visit `https://github.com/{owner}/{repo}/settings/hooks/new` to install a new
github webhook.

Enter this lambda's api gateway url.

Select Content type `application/json`

Enter this lambda's webhook secret

Select `Let me select individual events`

Select `Pull Requests`

Click `Add webook`

## 👩‍🏭 development

This is a [rustlang](https://www.rust-lang.org/en-US/) application.
Go grab yourself a copy of [rustup](https://rustup.rs/).

## 🚀 deployment

This is a rust application deployed using ⚡ [serverless](https://serverless.com/) ⚡.

> 💡 To install serverless, run `make dependencies`

This lambda is configured through its environment variables.

| Name                    | Description                                       |
|-------------------------|---------------------------------------------------|
| `GITHUB_TOKEN`          | token used to update github pull request          |
| `GITHUB_WEBHOOK_SECRET` | shared secret used to authenticate requests       |

> 💡 the `GITHUB_TOKEN` env var must have [repo scope](https://developer.github.com/apps/building-oauth-apps/understanding-scopes-for-oauth-apps/#available-scopes) in order to properly delete branches for the repository the webhook is configured
for.

Run `AWS_PROFILE=prod make deploy` to deploy.
# berg-cli

berg-cli is a helper cli to play ctfs running on the berg platform.

## Commands

**`berg-cli init <server> [path]`**

This initialises a berg repo by downloading all available challenges and attachments to the given path. This will use the current directory if path is not given.

**`berg-cli sync`**

Syncs the berg repo. This will download any new challenges it doesn't have yet.
If authenticated, it will also move any challenge folder that are completed to `.done/`

**`berg-cli authenticate`**

Authenticate the current repo. Requires a berg auth token.

## berg-repo

A berg repo in the context of this cli is a project folder with the following structure:

```
repo
|- .done/ folder including challenges that have been completed
|- category-a
   |- challenge-a
|- .berg.auth # berg auth file (optional)
|- .berg.toml # berg config file
|- .gitignore
```

## authentication

create a `.berg.auth` file in the root of your berg repo and insert the token from the berg-auth token you can find in your browser.

## todo

- flag submission command
- command to start challenges on the remote
- init should also initialise a git repository
- automatic grep for flag format + common encodings of flag format for forensic challenges
- spwn / pwninit for pwn challenges and maybe run an autopwn just in case

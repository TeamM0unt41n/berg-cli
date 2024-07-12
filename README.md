# berg-cli

berg-cli is a helper cli to play ctfs running on the berg platform.


### **`berg-cli init <server> [path]`**

This initialises a berg repo by downloading all available challenges and attachments to the given path. This will use the current directory if path is not given.

### **`berg-cli sync`**

Syncs the berg repo. This will download any new challenges it doesn't have yet.
If authenticated, it will also move any challenge folder that are completed to `.done/`. When used with the `--flagdump` flag, it will also try to submit any flags it finds in the challenge folders (`.flag` files).

### **`berg-cli submit <challenge> <flag>`**

Submits a flag for a given challenge. This will check if the flag is correct and if so, move the challenge to `.done/`.
In either case, it will cache the flag after submission to prevent resubmission.

### **`berg-cli instance start [challenge]`**

Starts an instance of a challenge. If no challenge is given, it will start an instance of the challenge in the current folder, if applicable.

### **`berg-cli instance stop`**

Stops the current challenge instance.

### **`berg-cli instance exploit <script>`**

This command is a WIP.

Exploits a challenge. This will start a challenge instance and exploit it.
There are various arguments to control instance lifecycle in the context of this command.

The script should be a python script configured to take the instance ip and port as arguments, eg: `python exploit.py $IP $PORT`

### **`berg-cli authenticate`**

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

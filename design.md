## Introduction
Simp v3 will be the third version of the **open (S)ecure (I)nstant (M)essaging (P)rotocol** (simp for short)
This time the protocol is written in the Rust programming language and then later the server and the clients are implemented like so in Rust.
There will be a complete specification manual for those who want to make their own clients compliant with the simp v3 protocol and server

## What features will v3 have?
The simp v3 server will have all the features that simp v2 and simp v1 had and also will mostly inherit the same handshake and post-kex communication process

### The features that will be inherited from v2 and v1 are:
####Commands
- `list` command
- `help` command

#### Handshake process
- Username and password credentials system
- RSA-4096 public key cryptograpghy with pkcs1v15 padding
- AES-GCM/256 encryption for all messages after kex

#### Username check
Usernames in simp v1 and v2 were checked whether they had fit in certain criterias:
- Usernames have to be Alphanumeric and cannot start with a number
- Usernames have to be at most 16 charachters

#### Credentials database
- In simp v1, v2 userdata was saved in a YAML file and was manipulated by the yamlcpp library
- In simp v3 we will use another file extention that is more friendlier to user in Rust and more secure

#### Prevent various bugs
Many bugs can occur in an encrypted instant messaging environment. Such bugs can include:
- Empty messages sent by clients
- Invalid packets sent by clients
- Incorrect handshake by clients
- Sending too many packets
- Invalid usernames
- Clients sending special characters like escape sequences (`\n`, `\r`, etc)
- Sneaking in divider bytes used by the protocol to exploit
-  Overflowing buffers

Many of these bugs were patched on the original simp v1 thanks to the `susrust` tool made by [vivy\_ir_](http://github.com/vivyir "vivy_ir_")
In simp v3 we hope to tackle all of these bugs with a new debugging tool and also taking advatange of Rust's memory safety

### The features that will be newly added to v3:
#### Seperate text channels
On default there will be a #main channel that every user is moved to upon joining, when connected to a channel, they will ONLY recieve:
- Incoming channel messages
- Server broadcasts
- Direct messages using /msg
- Command responses

and their messages are sent to the channel they are connected to
They can join new channels using `/join`

#### UUID
Every new user will have a unique UUID assigned to them, this allows easier manipulation of users with plugins and internal server logic and a cleaner protocol

#### Salted passwords
Passwords in the credentials database will be saved as hashes of [themselves with an added salt] that is stored right next to the other auth information of a specific user

#### New command engine
This command engine will have type annotations and some examples are shown from the "More commands" section of this text file

##### Example
we have the command `

    /tempmute <user: User> <duration: Time> [reason: Text]
You can either feed a command arguments by the order that is stated:


    /tempmute spixa 3d No reason
Or use any order you want using type annotations:


    /tempmute user:spixa reason:"No reason" duration:"3d"
These two will have the same result
This is inspired by Discord slash commands, except less clunky :)
##### Type annotations
Type annotations (AKA sub-command tags) are there to seperate arguments that have whitespaces in between them, like say in /msg spixa how are you?, you would want to add another argument, but how would you know when the content of the message you're sending to the user will finish?
Also third-party clients that are using fancy GUIs to say simplify the process of sending someone a private message using /msg will have an easier time implmenting such functionalities!

#### More commands
        /help <cmd: Command>: receive help about a specific command
        /msg <user: User> <msg: Text>: send a user a private messages
        /glist: global list of the server (/list will list the users in a certain channel)
        /reply <msg: Text>: reply to the latest user you interacted with
        /join <channel: Channel>: join a text channel
        /lock <channel: Channel>: lock a channel, prevent new users from joining
        /create <channel_name: String> [limit: u16] [password: String]: create a new text channel
		// ... and other channel related commands including moderation commands like /channelmute ...
        /kick <user: User> [reason: Text] : kick user
        /ban <user: User> [reason: Text] [duration: Time]: ban user
        /banip <ip: Ip/v4> [reason: Text]: ip-ban
        /mute <user: User> [reason: Text]: mute user
        /tempmute <user: User> <duration: Time> [reason: Text]
        /whitelist:
                <on/off>: toggle whitelist
                <add> <username: Username>: add a whitelisted username
                <remove> <username: WhitelistedUsername>: remove a whitelisted username
        /plugins: Wil list the plugins of the server
#### Plugins
Will be most likely written in Lua. Plugins can add commands and can listen to events that are evoked by the server itself

##### Example
```lua
function userMessageEvent(e: UserMessageEvent)
```
is a function that listens to the event of a new user message in a certain channel, inside this function we can implement different functionalities
For example, the user can be manipulated like so:
```lua
 if e.getMessageContent() == "shit" then e.getAuthor().kickUser("Inappropriate word!")
```
#### Permission levels and permission groups
In the user database, for each user there will be a "permissions" node which is a minecraft-like permission list. For example: (YAML example, we might switch to another ML language)
```yaml
82cfc07d-66a7-48cb-bb66-90b4066613e3:
		username: "spixa"
		password: "538b2144d7f55b7aee400550f2dd79a1fb3d80b3e06527c3bb0d09373423545d"
		salt: "A32CBFASJ7KA"
        permissions:
            core.basic.msg.*
            core.basic.help.*
            core.mod.kick.*
            core.mod.ban.temp.*
            core.mod.ban.perm
            core.mod.ban.perm.ops
           # ... and so on ...
```

### The features that might be added in the next versions of the protocol:
#### Per-server user profiles
#### CDN servers and user profile pictures
#### Fully encrypted voice channels with stream ciphers (ambitious)

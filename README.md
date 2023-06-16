# Name
Secure TCP, pretty simple!

# Why?
Y'know, it gets a bit annoying whenever you want to do networking and want it to be secure, but you have to write the encryption scheme for it from scratch, choosing algorithms etc etc, you get the gist, so now we have STCP! a more or less drop-in replacement for your secure networking needs (still indev, the plan is to make it as drop-in as just putting a `use` at the beginning of your program and forgetting about it!)

Please remember to look at the `examples/`, I'll try my best to add new ones and clear up the vagueness in current ones/refactor them!

# What's it use tho?
- RSA-4096 as the asymmetric encryption algorithm used for the handshake
- AES-256-GCM as the main symmetric encryption algorithm after the handshake
- Memory hardening for the RSA private key, even if someone were to hack your program and be skilled enough to find where the private key is located; they most likely will never be able to make it past the hardening, want to know how good the memory hardening is? then read on!

# How good is the memory hardening REALLY?
- The following is paraphrased from [hard's documentation][hard], the library used for memory hardening in `stcp`!

> This crate provides hardened buffer types backed by ***libsodium's secure memory management utilities***. The intention is to provide types for securely storing sensitive data (cryptographic keys, passwords, etc).
> Memory allocated using hard is placed directly at the end of a page, **followed by a guard page, so any buffer overflow will immediately result in the termination of the program.**

And if that by itself wasn't enough for you

> **A "canary" is placed before the allocated memory to detect modifications on free, and another guard page is placed before this.**

But if you're still not convinced

> The operating system is advised **not to swap the memory to disk, or include it in crash reports/core dumps.** Finally, when the memory is freed, **it is securely cleared, in such a way that the compiler will not attempt to optimise away the operation.**

I'd say that's better than even the tech some enterprise apps use, maybe ones sponsored by ***The State*** even (I'm looking at you, Iranian chat apps).

[hard]: https://docs.rs/hard/0.5.0/hard/

# Roadmap
- Make it better (no promises<3)

# License
All code in this repository (both original and all the pull requests by others) will be under MPL2.0, please enjoy<3

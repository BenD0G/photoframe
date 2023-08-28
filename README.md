# photoframe
For displaying a PCloud gallery on a screen.

# Notes on Building
It seems like the target we're aiming for is `armv7-unknown-linux-gnueabihf`. Cross-compilation only seems to work by using the `cross` tool and manually adding a dependency on `openssl` with a `vendored` feature (which compiles and statically links to its own `openssl`). Build with
```
cross build --release --target=armv7-unknown-linux-gnueabihf
```

Then run the ansible script:
```
ansible-playbook -i hosts.ini rpi_playbook.yaml -e PHOTOFRAME_USERNAME=$PHOTOFRAME_USERNAME -e PHOTOFRAME_PASSWORD=$PHOTOFRAME_PASSWORD -e PHOTOFRAME_OAUTH_TOKEN=$PHOTOFRAME_OAUTH_TOKEN
```
Where `hosts.ini` looks like
```
[raspberrypi]
192.168.xx.xxx ansible_ssh_user=pi ansible_ssh_pass=<pass>
```

# Cross-Compiling
Turns out, cross-compiling while linking to external libraries is a _nightmare_. As of this commit, I have something that works, and the only real downside is that I have to manually include an `openssl` crate in my dependencies. [This well-written blog post](https://capnfabs.net/posts/cross-compiling-rust-apps-raspberry-pi/) has some more tips and tricks that I'd like to try.

# PCloud OAuth Token
The PCloud docs are _completely insane_: they are inconsistent, often blatantly incorrect, always deeply aggravating.

What I've ended up with is this:
1. In your browser, navigate to https://eapi.pcloud.com/oauth2/authorize?client_id=CLIENT_ID&response_type=code, and copy the code.
2. Next, navigate to https://eapi.pcloud.com/oauth2_token?client_id=CLIENT_ID&client_secret=CLIENT_SECRET&code=CODE
3. Write down the access token and use that in your application.
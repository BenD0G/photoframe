# photoframe
For displaying a PCloud gallery on a screen.

# Auth
The PCloud docs are _completely insane_: they are inconsistent, often blatantly incorrect, always deeply aggravating.

What I've ended up with is this:
1. In your browser, navigate to https://eapi.pcloud.com/oauth2/authorize?client_id=CLIENT_ID&response_type=code, and copy the code.
2. Next, navigate to https://eapi.pcloud.com/oauth2_token?client_id=CLIENT_ID&client_secret=CLIENT_SECRET&code=CODE
3. Write down the access token and use that in your application.
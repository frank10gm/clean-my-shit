## Release a new version

Run from your own Terminal (so the notary keychain credential is reachable):

```sh
# 1. rebuild + re-sign + notarize
cd ~/workspace/hw/clean-my-shit
export DEVELOPER_ID_APP="$(security find-identity -v -p codesigning | grep 'Developer ID Application' | head -1 | sed -E 's/.*"(.*)"/\1/')"
export NOTARY_PROFILE=cms-notary
./packaging/macos/sign_and_notarize.sh

# 2. copy the signed .dmg into the website
cp dist/clean-my-shit.dmg ~/workspace/hw/hw-future/packages/website-and-cms/public/downloads/clean-my-shit.dmg

# 3. commit + redeploy the site
```

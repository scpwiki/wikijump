# User karma

- Feature ID: `karma`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Represent and display Wikidot user karma according to the documented visibility, progression, benefits, and anti-abuse behavior.

## Implementation contract

- The public route, UI, persistent state, permissions, and user-visible side effects MUST match the documented contract.
- Account, site, category, page, and actor context MUST be enforced at the public service boundary.
- Browser behavior MUST be tested when the feature exposes navigation, dynamic controls, or intermediate visible states.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Public HTTP route and browser-visible UI
- Public service/API boundary for persistent state and permissions

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:karma/source.wikidot.txt:1` through line 62 (canonical)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:80` through line 85 (supporting)

## Documentation-derived behavioral evidence

### doc:karma (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:karma/source.wikidot.txt:1` through line 62  
SHA-256 of complete source file: `b00adea177500787c6e65a8424e0bc7ae4f0dbc22d0649d314576e675de271cb`

```wikidot
L0001 ++ What is karma?
L0002 
L0003 Karma is a way to measure users' activity on Wikidot. Users with high karma are active in developing wikis and post frequently in wikidot forums.
L0004 
L0005 ++ How can I view someone's karma?
L0006 
L0007 Karma level is shown as a bar next to your avatar. The more bars you have the higher karma you have.
L0008 
L0009 ||~ Number of bars ||~ Icon ||~ Textual description ||
L0010 || 0 || [[image http://www.wikidot.com/common--theme/base/images/karma/karma_0.png]] || none ||
L0011 || 1 || [[image http://www.wikidot.com/common--theme/base/images/karma/karma_1.png]] || low ||
L0012 || 2 || [[image http://www.wikidot.com/common--theme/base/images/karma/karma_2.png]] || medium ||
L0013 || 3 || [[image http://www.wikidot.com/common--theme/base/images/karma/karma_3.png]] || high ||
L0014 || 4 || [[image http://www.wikidot.com/common--theme/base/images/karma/karma_4.png]] || very high ||
L0015 || 5 || [[image http://www.wikidot.com/common--theme/base/images/karma/karma_5.png]] || guru ||
L0016 
L0017 You can also see a member's karma by clicking on user's name. There you will find a users Karma along with the users standard profile.
L0018 
L0019 You can easily reach medium to high level karma through normal activity. However, the guru level is reserved for most active members.
L0020 
L0021 ++ How can I raise my karma?
L0022 
L0023 In general, the more active you are, the higher your karma is. You can get karma for various activities that include (but is not limited to):
L0024 
L0025 * creating and editing wiki content (pages)
L0026 * participating in forum discussions
L0027 * participating in open community portals (like [http://community.wikidot.com Wikidot's community wiki])
L0028 * being wiki administrator/moderator/member
L0029 * inviting other people to join Wikidot
L0030 * having contacts and friends on Wikidot
L0031 
L0032 ++ What are benefits of having high karma?
L0033 
L0034 The main benefit is that other users can instantly recognize you as an influential person and a wiki-expert.
L0035 
L0036 We are also going to give some extra functionality for top-karma users. For example, very high and guru users were invited to participate in beta tests of our pro accounts and after finishing those tests, they could buy pro accounts with a significant discount.
L0037 
L0038 ++ How is karma calculated?
L0039 
L0040 There are many factors in calculating a member's karma. Some of these are listed above. Only a 5-unit bar is displayed to make karma easy to see at a glance.[[footnote]] Guidelines for calculating karma have been included in the open source version. [[/footnote]]
L0041 
L0042 ++ Can someone vote for me so that I can get good karma?
L0043 
L0044 No. Karma is a reflection of your expertise, skill, and how helpful you are in forums. It is not intended to show the popularity of a member.
L0045 
L0046 [!--
L0047 ++ Why are the karma points not shown?
L0048 
L0049 Karma points are not shown not to give users a possibility to predict how exactly the karma calculation works and to eliminate cheating. Also we would not like getting karma to become a competition among users.
L0050 --]
L0051 
L0052 ++ I'm very active but my karma is low? What can I do?
L0053 
L0054 Editing pages is not the only factor taken to calculate karma. There are several factors that we take into account -- see above. If you want good karma, try helping members on the [http://community.wikidot.com/ community wiki]. This will raise your karma. [[footnote]] Karma is updated every few days and there might be a delay between your activity and your karma level. [[/footnote]]
L0055 
L0056 ++ I have high karma. Will it drop someday?
L0057 
L0058 The karma levels are recalculated every few days and your karma can go up or down. There is one exception: if you achieved __guru karma__ it will not drop. It is really hard to get to this level and we do not see any point in taking karma back from the gurus.
L0059 
L0060 ++ Can people cheat the karma system?
L0061 
L0062 We are constantly improving the algorithms to measure real activity of our members. We don't see any reason why anyone would cheat — even if someone could "cheat" to raise their karma, other members can easily verify his/her web activity and see if they are really active.
```

### features (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:80` through line 85  
SHA-256 of complete source file: `2f543ffe5d97f77da4936b7ab95ac66493b1acedd2bea01d5b956735b1b9501c`

```wikidot
L0080 +++ KARMA
L0081 Karma is an indicator of user's engagement and experience. It looks like a battery charge indicator :) It starts highlighting bar from the bottom (light green) to the top (red). The highest karma level (red) indicated that user is very advanced and experienced -- a Wikidot //Guru//. You can disable karma indicator if you don't want to show it or you can choose if it should be displayed on your Site (if you will set "no" -- all users on your site will not have the karma indicator displayed next to their avatar).
L0082 
L0083 
L0084 
L0085 
```

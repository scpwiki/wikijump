# Subscriptions and account/site plans

- Feature ID: `subscriptions-plans`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Represent Wikidot account and site upgrades, slots, storage limits, expiration, billing periods, administrator access, refunds, and payment rules.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:upgrades/source.wikidot.txt:1` through line 172 (canonical)

## Documentation-derived behavioral evidence

### faq:upgrades (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:upgrades/source.wikidot.txt:1` through line 172  
SHA-256 of complete source file: `91765a8ea2d1249a05cac88922ebb9fbe0db77790081984005b342bcf78a3f53`

```wikidot
L0001 +++ What upgrades can I buy?
L0002 
L0003 At Wikidot.com we have a very simple model for upgrades. Basically there are four simple upgrades available: 
L0004 
L0005 * **Pro Lite subscription**
L0006  * upgrades basic account functionality
L0007  * gives "pro" status to all your Sites, extending their features too
L0008  * space for 5 sites and 30GB of storage
L0009 * **Pro subscription**
L0010  * vastly extends functionality of your account
L0011  * gives "pro" status to all your Sites, extending their features too
L0012  * space for 10 sites and 100 GB storage
L0013 * **Pro+ subscription**
L0014  * all the functionality above and SSL (plus a few more features)
L0015  * includes space for 30 sites and 200 GB storage
L0016 * **Slots**
L0017  * provide additional space for new sites and storage
L0018  * 1 slot = 1 extra site + 5 GB storage.
L0019 
L0020 All upgrades are based on the **subscription model** -- you can buy upgrades for a time period, e.g. a year. You can extend your subscription for each of your upgrades at any time.
L0021 
L0022 The upgrades are **per User** instead of being "per Site". There are no additional fees for hosting each of your Sites within the limits of your account.
L0023 
L0024 +++ Is there a monthly / yearly fee for a "Pro" site?
L0025 
L0026 Instead of introducing a subscription fee for each of your sites, our plans are made simpler. The Pro Lite / Pro  / Pro+ upgrades are for a user's account and all user's Sites are promoted to Pro Lite / Pro / Pro+ status automatically. This is very different from what other wiki providers are offering and have a few key benefits:
L0027 * easier management of subscriptions
L0028 * much lower costs
L0029 
L0030 +++ What are the benefits of being a Pro / Pro+ subscriber?
L0031 
L0032 There are a lot of them. Among the most important are:
L0033 
L0034 * more Sites to create and more storage space
L0035 * SSL for all your Sites (available in Pro+ plan)
L0036 * higher limits of members for all your private Sites
L0037 * advanced web statistics for all your Sites
L0038 * configurable visibility of your profile
L0039 * no adverstising on your Sites
L0040 
L0041 You can compare the plans at http://www.wikidot.com/plans
L0042 
L0043 +++ How do I upgrade to the Pro account?
L0044 
L0045 The "Pro" plans are available as upgrades to the free account. 
L0046 * You can get a free account [http://www.wikidot.com/auth:newaccount here]. 
L0047 * The upgrade panel is available in the [https://www.wikidot.com/account:you#?start=upgrades "Upgrades" section in My Account].
L0048 
L0049 If you do not have a Pro account yet, you can upgrade directly to the Pro+ account as described above.
L0050 
L0051 If you do have the Pro account already, the Upgrade panel will show the option to upgrade to Pro+ plan. The Pro+ subscription will be valid for a year from the date of upgrade, but the price will be lowered by the unused period of your Pro account. 
L0052 
L0053 +++ How exactly do "slots" work?
L0054 
L0055 = **1 slot = space for 1 site + 1 GB storage**
L0056 
L0057 Simply speaking, a slot is a space for a new Site with space for the uploaded files.
L0058 
L0059 Slots can be purchased at the [https://www.wikidot.com/account:you Upgrade panel in My Account].
L0060 
L0061 +++ How do file storage and per-site limits work?
L0062 
L0063 With the free account, each of your Sites have 300 MB for file uploads. No more, no less. 
L0064 
L0065 In the Pro account it works differently. First of all, it is your account that has the storage. Now you can set maximum limits on how much of your storage can each of your Sites use.
L0066 
L0067 Let us say you have 10 GB from your Pro upgrade and 5 Sites. Now you could set 1 GB or 2 GB limits for each of the Sites, but you can also set 5GB or even no limits at all. In any of those cases the total files size cannot exceed the per-site limit, and the total used storage cannot exceed your account storage (10 GB in this case).
L0068 
L0069 So setting the per-site limits means: __use up to X GB from my account's storage__. Of course if your account runs out of free space, you will not be able to upload any new files.
L0070 
L0071 If you need more storage, each extra Slot gives you 1 GB.
L0072 
L0073 The system of account storage provides a very efficient (and cost-effective) storage space management.
L0074 
L0075 +++ What happens when my subscriptions expire?
L0076 
L0077 When your Pro subscription expires, your account and all your Sites will be downgraded to the free version. All the extra features will be downgraded. E.g. SSL will stop working, advanced web statistics will not be accessible anymore, your forum signature will be no longer displayed. No content however will be deleted upon downgrade and all the settings you had will be kept. Once you renew your Pro subscription, all features will be restored instantly.
L0078 
L0079 Number of slots decreases when either your Pro subscription expires or when your slot upgrades expire. When **number of slots drops** it might happen that the number of your Sites is larger than the number of your slots. In such a case the most recent Sites will be locked (read-only) until you prolong your subscription.
L0080 
L0081 If your file storage exceeds your limit after the downgrade, you will not be able to upload new files.
L0082 
L0083 +++ How much do the upgrades cost?
L0084 
L0085 Please refer to [http://www.wikidot.com/plans www.wikidot.com/plans] to learn more about the upgrades and pricing. 
L0086 
L0087 +++ Why so inexpensive?
L0088 
L0089 Our infrastructure (software and hardware) consists of top solutions in the industry, tuned to deliver very high performance and scales easily when needed. This help us reasonably manage our resources.
L0090 
L0091 Thanks to this policy, we are able to offer top-quality wikis at a price that is a fraction of the ones offered by our competitors, because we have a proven way to provide scalable, efficient and reliable infrastructure for the product at a balanced cost.
L0092 
L0093 +++ How long is my subscription?
L0094 
L0095 Both the Pro account and Slots are active for 1 year. When any of the features are about to expire, we will email you to remind you about it.
L0096 
L0097 We are also planning shorter periods for upgrades (monthly plans).
L0098 
L0099 +++ If I create new wiki __after__ buying Pro Account, will it be Pro too?
L0100 
L0101 Yes, all wikis created after buying Pro Account will automatically be Pro too.
L0102 
L0103 +++ I have a Pro Site with a few other guest Admins. Will they be able to configure Pro features?
L0104 
L0105 To configure settings (e.g. through the Site Manager) that come from the Pro package, the Admin also need to be a Pro user. Obviously you, as the Master Administrator of the given Site, have access to all settings, as well as other Admins with Pro accounts. Admins with free accounts can configure all non-Pro settings, but does not have acccess to custom domain settings, favicon etc.
L0106 
L0107 +++ Would anyone know that I am Pro User?
L0108 
L0109 It depends. By default Pro Accounts are marked by a Pro Icon shown next to the avatar. However there is an option to hide your Pro indicator for Pro+ Users. Moreover you can choose how your user information looks like by toggling displaying of karma, pro icons and avatars. The same way you can choose how your users icons will be displayed on each of your Sites. Both features are available only in Pro+ account.
L0110 
L0111 +++ Do you have a refund policy?
L0112 
L0113 Yes. We can refund any product purchased up to 30 days after the purchase. After 30 days all the sales are final and non-refundable. Any refund after 30 days is in our sole discretion.
L0114 
L0115 +++ What happens when you cannot provide the service I am paying for?
L0116 
L0117 If for some reason we cannot deliver the service, e.g. because major, prolonging server or network failure or software problems, in most circumstances we would prolong your subscriptions by the amount of time the service was not available.
L0118 
L0119 + Accounting and prices
L0120 
L0121 +++ What currencies do you accept?
L0122 
L0123 We are accepting payments in **USD** (United States Dollars) for countries outside of European Union and **Euro** for customers in European Union.
L0124 
L0125 Prices in Euro already contain VAT (Value Added Tax).
L0126 
L0127 +++ What are the payment methods?
L0128 
L0129 We accept credit card payments (VISA and MasterCard), PayPal payments, and bank transfer (for larger clients only).
L0130 
L0131 +++ Are payments safe?
L0132 
L0133 Yes, all the data is transferred using SSL secure channels with at least 128-bit encryption, which meets the industry standards for processing payments. We do our best to keep the data secure and impossible to misuse.
L0134 
L0135 +++ Do you issue VAT invoices?
L0136 
L0137 Yes, for every purchase a full VAT invoice is issued. You can use it for your accounting purposes (and VAT accounting if you are running a business in European Union).
L0138 
L0139 +++ What is VAT?
L0140 
L0141 VAT, Value Added Tax,  is a consumption tax levied on value added. Please find more information in this [http://en.wikipedia.org/wiki/Taxation_in_the_European_Union Wikipedia article about VAT in European Union].
L0142 
L0143 +++ Who is charged with VAT on purchases and how much?
L0144 
L0145 All non-business customers from European Union (EU), EU business clients  without valid EU VAT ID and all clients in Poland are charged 23% VAT. 
L0146 
L0147 Companies in European Union with a valid EU VAT ID and customers (both individual and companies) from outside of European Union are not charged VAT.
L0148 
L0149 All customers from Poland are charged VAT.
L0150 
L0151 ||~ Who? ||~ Charged VAT? ||
L0152 || Individual not in EU || No ||
L0153 || Business not in EU || No ||
L0154 || Individual in EU || Yes ||
L0155 || Business in EU with valid EU VAT ID || No ||
L0156 || Business in EU without valid EU VAT ID || Yes ||
L0157 || Individual and Business in Poland || Yes || 
L0158 
L0159 We know it is a bit complicated, but this is how VAT works. We made the upgrade panel automatically handle all those cases so you do not have to worry about it. When you upgrade, you will be presented with prices either with or without VAT.
L0160 
L0161 +++ When I enter my EU VAT ID the error says it is not valid.
L0162 
L0163 Wikidot Inc. automatically verifies validity of EU VAT numbers using the [http://ec.europa.eu/taxation_customs/vies/vieshome.do VIES] service but take no responsibility for outdated or false results provided by VIES. The top reasons your VAT ID is not valid according to VIES might be:
L0164 * The VAT ID is a national VAT ID and not a EU VAT ID (this is a difference)
L0165 * The VIES database is not up-to-date (see their [http://ec.europa.eu/taxation_customs/vies/faqvies.do#item12 FAQ]
L0166 * Communication error with the VIES database has occurred.
L0167 
L0168 If VIES database is not validating your VAT ID properly and you still want to qualify for VAT reverse charge (i.e. you do not want VAT to be included in your prices), please contact us at sales@wikidot.com. Most likely we will ask you to fax your VAT registration documents to us and we will manually approve your VAT number.
L0169 
L0170 +++ I have a national VAT ID (not a EU VAT ID) but still want it to appear on my invoice.
L0171 
L0172 In such a case please put your VAT ID in a field other than the EU VAT ID, e.g. put it next to the Company's name, e.g. "Wiki LLC, VAT ID: 0987654321"
```

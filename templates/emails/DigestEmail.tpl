Hi, {$user->getNickName()|escape}!

{if $count==1}There is one notification{else}There are {$count} notifications{/if} related to your Wikidot account:
To view the full list of notifications visit:
http://{$URL_HOST}/account:you/start/notifications
{foreach from=$notifications item=notification}

{$notification->getTitle()}

{$notification->getBody()|replace:'<br/>':"\n"|strip_tags}
{assign var=extra value=$notification->getExtra()}{if $extra && $extra.urls}{foreach from=$extra.urls item=url}{$url[0]}: {$url[1]}
{/foreach}{/if}
{/foreach}

If you do not wish to receive such notification digests please go to
http://{$URL_HOST}/account:you/start/settings [you account settings]
and configure the Notifications section.
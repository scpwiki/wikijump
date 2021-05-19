<h1>{t}Member of...{/t}</h1>

{if isset($memberships)}
	<div class="sites-list">
		{foreach from=$memberships item=member}
			<div class="site-list-item">
				{assign var=site value=$member->getSite()}
				<div class="options">
					{t}options{/t}: <a href="javascript:;" onclick="Wikijump.modules.AccountMemberOfModule.listeners.signOff(event, [{$site->getSiteId()}, '{$site->getName()}'])">{t}sign off{/t}</a>
				</div>
				<div class="name">
					<a href="{$HTTP_SCHEMA}://{$site->getDomain()}">{$site->getName()|escape}</a>
				</div>
				{if $site->getSubtitle()}
					<div class="subtitle">
						{$site->getSubtitle()|escape}
					</div>
				{/if}
				{if $site->getDescription()}
					<div class="description">
						{$site->getDescription()|escape}
					</div>
				{/if}

			</div>
		{/foreach}
	</div>
{else}
	{t}Currently you are not a member of any site :-({/t}
{/if}
<div style="display: none" id="signoff-window">
	{t escape=no}Are you sure you do not want to be a member of the site <strong>%%SITE_NAME%%</strong> any more?<br/>
	If you have any additional role in this site (admin, moderator) it will be lost too.{/t}
</div>

<h1>{t}Moderator of...{/t}</h1>

{if isset($moderators)}
	<div class="sites-list">
		{foreach from=$moderators item=moderator}
			<div class="site-list-item">
				{assign var=site value=$moderator->getSite()}
				<div class="options">
					{t}options{/t}: <a href="javascript:;" onclick="Wikijump.modules.AccountModeratorOfModule.listeners.resign(event,{$site->getSiteId()}, '{$site->getName()|escape}')">{t}resign{/t}</a>
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
	{t}Currently you are not a moderator of any site :-({/t}
{/if}

<div id="moderator-resign-dialog" style="display: none">
	{t}Are you sure you want to resign from being a site moderator of
	<strong>%%SITE_NAME%%</strong>?{/t}
</div>

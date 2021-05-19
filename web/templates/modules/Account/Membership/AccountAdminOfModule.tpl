<h1>{t}Admin of...{/t}</h1>

{if isset($admins)}
	<div class="sites-list">
		{foreach from=$admins item=admin}
			<div class="site-list-item">
				{assign var=site value=$admin->getSite()}
				<div class="options">
					{t}options{/t}: <a href="javascript:;" onclick="Wikijump.modules.AccountAdminOfModule.listeners.resign(event,{$site->getSiteId()}, '{$site->getName()|escape}')">{t}resign{/t}</a>
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
	{t}Currently you are not an administrator of any site :-({/t}
{/if}


<div id="admin-resign-dialog" style="display: none">
	{t}Are you sure you want to resign from being a site administrator of{/t}
	<strong>%%SITE_NAME%%</strong>?
</div>

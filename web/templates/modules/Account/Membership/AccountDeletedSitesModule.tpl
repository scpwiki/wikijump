<h1>{t}Deleted Sites{/t}</h1>

{if isset($admins)}
	<div class="sites-list">
		{foreach from=$admins item=admin}
			<div class="site-list-item">
				{assign var=site value=$admin->getSite()}
				<div class="options">
					{t}options{/t}: <a href="javascript:;" onclick="Wikijump.modules.AccountDeletedSitesModule.listeners.clickRestore(event,{$site->getSiteId()})">{t}restore{/t}</a>
				</div>
				<div class="name">
					{$site->getName()|escape}
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
				<div>
					previously located at: {$HTTP_SCHEMA}://{$site->getUnixName()|regex_replace:"/\.\.del\.\..*$/":""}.{$URL_DOMAIN}
				</div>

			</div>
		{/foreach}
	</div>


{else}
	{t}You have no deleted sites. To delete any site of yours you could go to the Site Manager of a given site and look under <em>Extreme Actions</em> menu.{/t}
{/if}

<div id="as-restore-site-box" style="display:none">
	<h1>Restore site <em id="as-restore-site-name"></em></h1>

	<p>
		Great, you are ready to restore this previously deleted wiki. The site should look like and work exactly as before deleting it.
	</p>
	<p>
		You could also set a new URL address for this recevered site.
	</p>

	<form onsubmit="return false;">
		<table class="form">
			<tr>
				<td>
					Website base URL:
				</td>
				<td>
					<input type="text" class="text" size="20" id="as-restore-site-unixname" style="text-align: right"/>.{$URL_DOMAIN}
				</td>
			</tr>
		</table>
		<div class="buttons">
			<input type="button" value="cancel" onclick="$('as-restore-site-box').style.display='none'"/>
			<input type="button" value="restore" onclick="Wikijump.modules.AccountDeletedSitesModule.listeners.restore(event)"/>
		</div>
	</form>
</div>

<div id="as-restore-site-data" style="display:none;">
{$sitesData|escape}
</div>

<h1>Change base URL (rename)</h1>

<p>
	Current base URL address of this site is: <strong>{$site->getUnixName()}</strong>.{$URL_DOMAIN}. If you want to change it (i.e. the part before
	the ".{$URL_DOMAIN}" domain, please use the following form.
</p>
<p>
	Please note however that this might confuse your users since they will not be able to access this wiki with the old web address.
</p>
<p>
	This action is available only to the person who actually
	started the Site (the founder).
</p>
<p>
	Once again: please, use it with care.
</p>

{if isset($allowed)}
	<form onsubmit="return false;">
		<table class="form">
			<tr>
				<td>
					Website base URL:
				</td>
				<td>
					<input type="text" class="text" size="20" id="sm-rename-site-unixname" style="text-align: right" value="{$site->getUnixName()}"/>.{$URL_DOMAIN}
				</td>
			</tr>
		</table>
		<div class="buttons">
			<input type="button" value="change URL" onclick="Wikijump.modules.ManagerSiteRenameModule.listeners.renameSite(event)"/>
		</div>
	</form>
{else}
	<div class="error-block">
		Sorry, this option is available only to the founder of this site - {printuser user=$founder image=true}
	</div>
{/if}

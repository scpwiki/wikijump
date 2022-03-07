<h1>{t}Your very own custom domain{/t}</h1>

<p>
	Each Wiki Site obtains a web address in a dedicated <em>{$URL_DOMAIN}</em> subdomain.
	However, it is possible to use an external domain to reach this Site.
</p>
<p>
	If you enable this feature this Site will be available via both domains, i.e.
	<em>{$site->getSlug()|escape}.{$URL_DOMAIN}</em> <u>and</u> the new domain.
</p>
<p>
	You can also set up to 3 <strong>"301 redirects"</strong>. This means that if these domain
	are handled by the {$SERVICE_NAME} servers your visitors will be redirected to the main
	domain for the Site. This is useful, for example, to handle domains without the "www." prefix.
</p>
<p>
	Warning: this is a cool feature and totally free!
</p>

<div class="error-block" id="sm-domain-error" style="display: none"></div>

<form id="sm-domain-form">
	<table class="form">
		<tr>
			<td>
				{t}Custom domain{/t}:
			</td>
			<td>
				<input class="text" id="sm-domain-field" type="text" value="{$site->getCustomDomain()|escape}" size="40" maxlength="50"/>
				<div class="sub">
					{t}E.g. www.example.com{/t}
				</div>
			</td>
		</tr>
		<tr>
			<td>
				{t}301 redirects{/t}:
			</td>
			<td>
				{*
				<input style="margin: 3px 0;" class="text" id="sm-domain-redirect-1" value="{$redirects[0]}" type="text" value="" size="40" maxlength="50"/>
				<br/>
				<input style="margin: 3px 0;" class="text" id="sm-domain-redirect-2" value="{$redirects[1]}" type="text" value="" size="40" maxlength="50"/>
				<br/>
				<input style="margin: 3px 0;" class="text" id="sm-domain-redirect-3" value="{$redirects[2]}" type="text" value="" size="40" maxlength="50"/>
				<div class="sub">
					{t}E.g. example.com (optional){/t}
				</div>
				*}
				<div id="sm-redirects-box">
					{if $redirects}
						{foreach from=$redirects item=redirect}
							<div>
								<input style="margin: 3px 0;" class="text"  value="{$redirect}" type="text" value="" size="40" maxlength="50"/>
								<a href="javascript:;" onclick="Wikijump.modules.ManagerSiteDomainModule.listeners.removeRedirect(event)">remove</a>
							</div>
						{/foreach}
					{else}
						<div>
							<input style="margin: 3px 0;" class="text"  value="" type="text" value="" size="40" maxlength="50"/>
							<a href="javascript:;" onclick="Wikijump.modules.ManagerSiteDomainModule.listeners.removeRedirect(event)">remove</a>
						</div>
					{/if}
				</div>
				<a href="javascript:;" onclick="Wikijump.modules.ManagerSiteDomainModule.listeners.addRedirect(event)">+ add more</a>
				<div id="sm-redirect-template" style="display:none">
					<input style="margin: 3px 0;" class="text"  value="" type="text" value="" size="40" maxlength="50"/>
					<a href="javascript:;" onclick="Wikijump.modules.ManagerSiteDomainModule.listeners.removeRedirect(event)">remove</a>
				</div>
			</td>

		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" id="sm-domain-cancel"/>
		<input type="button" value="{t}clear mapping{/t}" id="sm-domain-clear"/>
		<input type="button" value="{t}save changes{/t}" id="sm-domain-save"/>
	</div>
</form>

<h2>What should you do <u>before</u> setting a custom domain?</h2>

<p>
	In order for the domain mapping to work you should also do a few things:
</p>
<ul>
	<li>
		You should own (or have administrative access to) the domain you want to use,
	</li>
	<li>
		You should point your nameservers (via "control panel" if your provider gives you one -
		look for advanced DNS settings -  or
		by any other means) to resolve your domain to servers of {$SERVICE_NAME}. This should be done
		by setting a <b>CNAME record</b> to "<b>{$URL_DOMAIN}</b>" value.
	</li>
	<li>
		Very often one has to wait (depends on your provider or DNS settings) for the changes
		to propagate over the internet.
	</li>
	<li>
		Exactly the same procedure should be applied for the redirected domains (URLs) to
		be handled by Wikijump servers.
	</li>
	<li>Sometimes your might be forced to provide the IP address
		of the Wikijump server. In such a case use <b>{$IP_HOST}</b>.
	</li>
</ul>

<h2>Note:</h2>
<p>
	This site will <u>always</u> be available via <em>{$HTTP_SCHEMA}://{$site->getSlug()|escape}.{$URL_DOMAIN}</em> address
	regardless of the custom domain settings.
</p>
<p>
	Any changes in the redirections might take up to a few minutes before becoming effective, due to caching.
</p>

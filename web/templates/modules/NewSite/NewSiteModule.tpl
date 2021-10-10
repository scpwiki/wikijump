<div id="new-site-box">
	{if $notLogged}
		<h3>We are almost ready to create a new site for you{if $unixName} at <span style="text-decoration: underline">{$unixName}.{$URL_DOMAIN}</span>{/if}...</h3>

		<p>
			{t 1=$SERVICE_NAME}However you would need to have a valid user account at %1 so that we could identify you in the future.{/t}
		</p>
		<table style="margin: 1em auto">
			<tr>
				<td style="text-align: center; padding: 1em">
					<div style="font-size: 180%; font-weight: bold;">
						<a href="javascript:;" onclick="Wikijump.page.listeners.loginClick(event)"
							>{t}Log in{/t}</a>
					</div>
					<p>
						{t}if you already have a {$SERVICE_NAME} account{/t}
					</p>
				</td>
				<td style="padding: 1em; font-size: 140%">
					{t}or{/t}
				</td>
				<td style="text-align: center; padding: 1em">
					<div style="font-size: 180%; font-weight: bold;">
						<a href="javascript:;"  onclick="WIKIREQUEST.createAccountSkipCongrats=true;Wikijump.page.listeners.createAccount(event)"
							>{t}Get a new account{/t}</a>
					</div>
				</td>
			</tr>
		</table>

	{else}


		<div class="error-block" id="new-site-form-errors" style="display: none"></div>

		<form id="new-site-form">
			<table class="form">
				<tr>
					<td>
						{t}Site name{/t}:
					</td>
					<td>
						<input class="text" type="text" id="new-site-name" name="name" size="30" value="{$siteName|escape}" />
						<div class="sub">
							{t}Appears on the top-left corner of your {$SERVICE_NAME} site.{/t}
						</div>
					</td>
				</tr>
				<tr>
					<td>
						{t}Tagline{/t}:
					</td>
					<td>
						<input class="text" type="text" name="subtitle" size="30" />
						<div class="sub">
							{t}Appears beneath the name.{/t}
						</div>
					</td>
				</tr>
				<tr>
					<td>
						{t}Web address{/t}:
					</td>
					<td>
						<input class="text" type="text" id="new-site-unixname" name="unixname" size="20" style="text-align: right" value="{$unixName|escape}"/>.{$URL_DOMAIN}
						<div class="sub">
							{t}Only alphanumeric [a-z0-9] and "-" (dash) characters allowed.{/t}
						</div>
					</td>
				</tr>
				{*<tr>
					<td>
						{t}Site content language{/t}:
					</td>
					<td>
						<input type="radio" name="language" value="en" id="new-site-lang-en"> <label for="new-site-lang-en">{t}English{/t}</label>
						&nbsp;&nbsp;&nbsp;&nbsp;
						<input type="radio" name="language" value="pl" id="new-site-lang-pl"> <label for="new-site-lang-pl">{t}Polish{/t}</label>

					</td>
				</tr>*}
				<tr>
					<td>
						{t}Initial template{/t}:
					</td>
					<td>
						<select name="template">
							<option value="">{t}-- please select initial layout for your wiki --{/t}</option>
							{foreach from=$templates item=template}
								<option value="{$template->getSiteId()}">{$template->getName()|escape}</option>
							{/foreach}
						</select>
					</td>
				<tr>
					<td>
						{t}Private site?{/t}
					</td>
					<td>
						<input type="checkbox" name="private" class="checkbox">
						<div class="sub">
							{t}If you check this, the site is visible only to its members.{/t}
						</div>
					</td>

				</tr>
				<tr>
					<td>
						{t}Please confirm:{/t}
					</td>
					<td>
						<input type="checkbox" name="tos" class="checkbox">
						<br/>
						{t 1=$URL_HOST}I have read and agree to the <a href="{$HTTP_SCHEMA}://%1/legal:terms-of-service"
						target="_blank">Terms of Service</a>.{/t}

					</td>

				</tr>
			</table>
			<div class="buttons">
				<input type="button" value="{t}Get a new wiki{/t}" onclick="Wikijump.modules.NewSiteModule.listeners.next3(event)"/>
			</div>
		</form>
	{/if}
</div>

<h1>OpenID Identities</h1>

<p>
	According to <a href="http://openid.net">OpenID.net</a>:
</p>
<blockquote>
	<p>
		OpenID is an open, decentralized, free framework for user-centric digital 
		identity.
	</p>
	<p>
		OpenID starts with the concept that anyone can identify themselves on the 
		Internet the same way websites do-with a URI (also called a URL or web address).
		Since URIs are at the very core of Web architecture, they provide a solid 
		foundation for user-centric identity.
	</p>	
</blockquote>

<p>
	This is a testing feature and we assume that you already know how OpenID works and how it can be used.
	At Wikidot we are working towards providing true OpenID server and accepting OpenID
	logins, but at the moment we thoght it would be cool to allow you to use your 
	Wiki URL as an OpenID login.
</p>

<p>
	How it works? You must already have a OpenID account at one of the identiy providers 
	(see below). When you try to log into any OpenID-enabled services, your 
	Wikidot Wiki will simply redirect (delegate) to your id provider.
</p>

<form>
	
	<table class="form">
		<tr>
			<td>
				Enable OpenID for this Wiki:
			</td>
			<td>
				<input id="sm-openid-enable" type="checkbox" {if $enabled}checked="checked"{/if}/>
			</td>
		</tr>
	</table>	
</form>

<hr/>

<div id="sm-openid-entry-0">
	<form id="sm-openid-form-0">
		<table class="form" style="margin: 1em auto 1em 0;">
			<tr>
				<td>
					Page:
				</td>
				<td>
					"root" (default) page
				</td>
			</tr>
			<tr>
				<td>
					Effective URL:
				</td>
				<td>
					http://{$siteDomain}
				</td>
			</tr>
			<tr>
				<td>
					Type:
				</td>
				<td>
					<select>
						<option>delegation</option>
					</select>
				</td>
			</tr>
			<tr>
				<td>
					URL of the identity:
				</td>
				<td>
					http://<input name="identityUrl" id="sm-openid-urlid-0" class="text" 
								type="text" size="34" maxlength="70" 
								value="{if $openIdRoot && $openIdRoot->getUrl()}{$openIdRoot->getUrl()|replace:"http://":""}{/if}"   
								onchange="WIKIDOT.modules.ManageSiteOpenIDModule.listeners.onIdentityChange(event, 0)"
							/>
				</td>
			</tr>
			<tr>
				<td>
					URL of the server:
				</td>
				<td>
					<input name="serverUrl" id="sm-openid-urlserver-0" 
						class="text" type="text" size="40" maxlength="70" 
						value="{if $openIdRoot && $openIdRoot->getServerUrl()}{$openIdRoot->getServerUrl()}{else}http://{/if}"
						
					/>
				</td>
			</tr>
		</table>
	</form>
</div>
	
<div id="sm-openid-idblock">
	{assign var=count value=1}
	{foreach from=$openIds  item=oo}
		<div id="sm-openid-entry-{$count}">
			<form id="sm-openid-form-{$count}">
				<table class="form"  style="margin: 1em auto 1em 0;">
					<tr>
						<td>
							Page:
						</td>
						<td>
							<div class="autocomplete-container" style="width: 20em">
								<input name="page" type="text" id="sm-openid-p-{$count}" class="autocomplete-input text" name="default_page" size="10" 
								value="{$oo->getPageUnixName()}" 
								onchange="$('sm-openid-effp-{$count}').innerHTML=(this.value)"/>
								<div id="sm-openid-p-list-{$count}" class="autocomplete-list"></div>
							</div>
							<div class="sub">
								
							</div>
						</td>
					</tr>	
					<tr>
						<td>
							Effective URL:
						</td>
						<td>
							http://{$siteDomain}/<span id="sm-openid-effp-{$count}">{$oo->getPageUnixName()}</span>
						</td>
					</tr>
					<tr>
						<td>
							Type:
						</td>
						<td>
							<select>
								<option id="sm-openid-type-{$count}">delegation</option>
							</select>
						</td>
					</tr>
					<tr>
						<td>
							URL of the identity:
						</td>
						<td>
							http://<input name="identityUrl" id="sm-openid-urlid-{$count}" 
							class="text" type="text" size="34" maxlength="70" 
							value="{$oo->getUrl()|replace:"http://":""}" 
							onchange="WIKIDOT.modules.ManageSiteOpenIDModule.listeners.onIdentityChange(event, {$count})"/>
						</td>
					</tr>
					<tr>
						<td>
							URL of the server:
						</td>
						<td>
							<input name="serverUrl" id="sm-openid-urlserver-{$count}" 
							class="text" type="text" size="40" maxlength="70" 
							value="{$oo->getServerUrl()}"/>
						</td>
					</tr>
					<tr>
						<td>
							Action:
						</td>
						<td>
							<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteOpenIDModule.listeners.deleteEntry(event, {$count})">delete entry</a>
						</td>
					</tr>
				</table>
			</form>
		</div>
		{assign var=count value=$count+1}
	{/foreach}
</div>
<div style="text-align: center">
	<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteOpenIDModule.listeners.addEntry(event)">+ Add another entry</a>
</div>
	
<div id="sm-openid-templateform" style="display: none;">
	<form id="sm-openid-form-RAND">
		<table class="form"  style="margin: 1em auto 1em 0;">
			<tr>
				<td>
					Page:
				</td>
				<td>
					<div class="autocomplete-container" style="width: 20em">
						<input name="page" type="text" id="sm-openid-p-RAND" class="autocomplete-input text" name="default_page" size="10" value="" onchange="$('sm-openid-effp-RAND').innerHTML=(this.value)"/>
						<div id="sm-openid-p-list-RAND" class="autocomplete-list"></div>
					</div>
					<div class="sub">
						
					</div>
				</td>
			</tr>	
			<tr>
				<td>
					Effective URL:
				</td>
				<td>
					http://{$siteDomain}/<span id="sm-openid-effp-RAND"></span>
				</td>
			</tr>
			<tr>
				<td>
					Type:
				</td>
				<td>
					<select>
						<option id="sm-openid-type-RAND">delegation</option>
					</select>
				</td>
			</tr>
			<tr>
				<td>
					URL of the identity:
				</td>
				<td>
					http://<input name="identityUrl" id="sm-openid-urlid-RAND" class="text" type="text" size="34" maxlength="70" value="" onchange="WIKIDOT.modules.ManageSiteOpenIDModule.listeners.onIdentityChange(event, RAND)"/>
				</td>
			</tr>
			<tr>
				<td>
					URL of the server:
				</td>
				<td>
					<input name="serverUrl" id="sm-openid-urlserver-RAND" class="text" type="text" size="40" maxlength="70" value="http://"/>
				</td>
			</tr>
			<tr>
				<td>
					Action:
				</td>
				<td>
					<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteOpenIDModule.listeners.deleteEntry(event, RAND)">delete entry</a>
				</td>
			</tr>
		</table>
	</form>
</div>

<div class="buttons">
	<input type="button" value="{t}cancel{/t}" onclick="WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-welcome')"/>
	<input type="button" value="{t}save{/t}" onclick="WIKIDOT.modules.ManageSiteOpenIDModule.listeners.save(event)"/>
</div>

<div id="sm-openid-patterns" style="display: none;">
	{$openIdServices}
</div>

<p>
	The delegation works fine with the following identity providers:
</p>
<ul>
	<li><a href="http://www.myopenid.com">myopenid.com</a></li>
</ul>
	
		
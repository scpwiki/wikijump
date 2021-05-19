{if isset($allowHttp)}
<h1>Secure Access (SSL/TLS)</h1>

<p>
	Configure secure access via <b>HTTP<span style="color: red;">S</span></b>:// connection.
	If you care about privacy and security - you should.
</p>

<form>
	<table class="form">
		<tr>
			<td>
				{t}Secure access mode{/t}:
			</td>
			<td>
				<select id="sm-ssl-mode-select">
					<option value="" {if $secureMode==''}selected="selected"{/if}>disabled</option>
					<option value="ssl" {if $secureMode=='ssl'}selected="selected"{/if}>both unsecure and SSL enabled</option>
					<option value="ssl_only" {if $secureMode=='ssl_only'}selected="selected"{/if}>SSL only (recommended for private sites)</option>
				</select>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" onclick="Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-welcome')"/>
		<input type="button" value="{t}save{/t}" onclick="Wikijump.modules.ManageSiteSecureAccessModule.listeners.save(event)"/>

	</div>
</form>

<p>
	The secure mode (SSL/TSL via HTTPS) means that the whole
	connection between your web browser and your Wiki Site is encrypted. No chance someone
	could intercept your transmission. This is very useful for private (non-public) sites.
</p>

<p>
	<b>NOTE:</b> The embedded code (via <tt>[[embed]]...[[/embed]]</tt> tags) may sometimes
	produce a warning of <em>non-secure elements</em> or <em>partially encrypted content</em>
	in the page if you are using SSL. This warning can also be triggered when you include other elements
	(like images, iframes) from non-encrypted sources (urls starting with http://, not http<b>s</b>://).
</p>
<p style="font-size: 120%; font-weight: bold;">
	In case of troubles - please contact <a href="mailto:{$SUPPORT_EMAIL}">{$SUPPORT_EMAIL}</a>.
</p>
{/if}

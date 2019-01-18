<h1><a href="javascript:;" onclick="WIKIDOT.modules.AccountModule.utils.loadModule('am-settings')">{t}Account settings{/t}</a> / {t}Language settings{/t}</h1>

<p>
	{t escape=no}Now it is possible to choose your preferred language. This setting will affect the language
	that <em>my account</em> interface will use and a few other things.
	However the interface within particular
	Sites will be displayed in the Sites' languages.{/t}
</p>

<form>
	<table class="form">
		<tr>
			<td>
				{t}Language{/t}:
			</td>
			<td>
				<select id="as-language-select">
					<option value="en" {if $lang=="en"}selected="selected"{/if}>{t}English{/t}</option>
					<option value="pl" {if $lang=="pl"}selected="selected"{/if}>{t}Polish{/t}</option>
				</select>	
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" onclick="WIKIDOT.modules.AccountModule.utils.loadModule('am-settings')"/>
		<input type="button" value="{t}save{/t}" onclick="WIKIDOT.modules.ASLanguageModule.listeners.save(event)"/>
	</div>
</form>
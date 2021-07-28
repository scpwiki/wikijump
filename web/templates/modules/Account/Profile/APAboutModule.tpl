<h1><a href="javascript:;" onclick="Wikijump.modules.AccountModule.utils.loadModule('am-profile')">{t}My profile{/t}</a> / {t}About myself{/t}</h1>

<p>
	{t}The information below is optional but might help.{/t}
</p>
<p>
	 {t}Note: Each item entered here will be visible to the public.
	 Please do not enter any information you do not want to disclose.{/t}
</p>

<div>
	<form id="about-form">
		<table class="form">
			<tr>
				<td>{t}Real name{/t}:</td>
				<td><input class="text" name="real_name" type="text" size="40" maxlength="80" value="{$profile->real_name|escape}"/></td>
			</tr>
			<tr>
				<td>Pronouns:</td>
				<td>
                    <input class="text" name="pronouns" type="text" size="40" maxlength="30" value="{$profile->pronouns|escape}"/>
				</td>
			</tr>

			<tr>
				<td>{t}Shortly about myself{/t}:</td>
				<td>
					<textarea id="about-textarea" name="about" cols="40" rows="5">{$profile->bio|escape}</textarea>
					<div class="sub">
						<span id="chleft">200</span> {t}characters left{/t}<br/>
						{t}This is a short description shown when someone clicks on your name
						everywhere it appears.{/t}
					</div>
				</td>
			</tr>
		</table>

		<h2>{t}My online presence{/t}</h2>

		<table class="form">
			<tr>
				<td>
					{t}My website{/t}:
				</td>
				<td>
					<input  class="text" name="website" type="text" size="40" maxlength="50" value="{$profile->about_page|escape}"/>
					<div class="sub">
						{t}Please start with the <em>http://</em>{/t}
					</div>
				</td>
			</tr>
		</table>
		<div class="buttons">
			<input type="button" value="{t}cancel{/t}" onclick="Wikijump.modules.AccountModule.utils.loadModule('am-profile')"/>
			<input type="button" value="{t}save{/t}" onclick="Wikijump.modules.APAboutModule.listeners.save(event)"/>
		</div>
	</form>
</div>

<div class="error-block" id="edit-theme-error" style="display: none"></div>

<form id="sm-edit-theme-form">
	<table class="form">
		<tr>
			<td>
				Theme name:
			</td>
			<td>
				<input  class="text" type="text" name="name" size="40" value="{if isset($theme)}{$theme->getName()|escape}{/if}"/>
			</td>
		</tr>
		<tr>
			<td>
				Which theme to extend:
			</td>
			<td>
				<select name="parentTheme">
					{foreach from=$exthemes item=extheme}
						<option value="{$extheme->getThemeId()}" {if $theme && $theme->getExtendsThemeId() == $extheme->getThemeId()}selected="selected"{/if} >{$extheme->getName()|escape}</option>
					{/foreach}
				</select>
				<div class="sub">
					Choose <em>Base</em> to create as much as possible from scratch.
				</div>
			</td>
		</tr>
		<tr>
			<td colspan="2">
				<p>CSS code:</p>
				<textarea id="sm-csscode" name="code" rows="15" cols="50" style="width: 100%">{$code|escape}</textarea>

			</td>
		</tr>
		<tr>
			<td colspan="2">
				<p>
					You can also edit the code on one of the wiki pages. This is
					recommended.<br/>
					Enter the page name below to import the CSS code.
				</p>
				<div class="error-block" id="cssimport-error" style="display: none"></div>
				<div style="text-align: center">
					<div class="autocomplete-container" style="width: 20em; margin: 0 auto;">
						<input type="text" id="sm-cssimport-input" class="autocomplete-input text" name="cssImportPage" size="35"
							value="{if isset($theme)}{$theme->getSyncPageName()|escape}{/if}"/>
						<div id="sm-cssimport-input-list" class="autocomplete-list"></div>
					</div>
					<br/>
					<input class="button" type="button" value="import" onclick="Wikijump.modules.ManageSiteCustomThemesModule.listeners.importCss(event)"/>
				</div>



			</td>
		</tr>
		<tr>
			<td>
				Use top menu bar:
			</td>
			<td>
				<input type="checkbox" name="useTopBar" {if ($theme && $theme->getUseTopBar()) || !$theme}checked="checked"{/if}/>
			</td>
		</tr>
		<tr>
			<td>
				Use side menu bar:
			</td>
			<td>
				<input type="checkbox" name="useSideBar" {if ($theme && $theme->getUseSideBar()) || !$theme}checked="checked"{/if}/>
				<div class="sub">
					If unchecked - these navigation elements will not be rendered.
				</div>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="cancel" onclick="Wikijump.modules.ManageSiteCustomThemesModule.listeners.cancelEditTheme(event)"/>
		<input type="button" value="save theme"  onclick="Wikijump.modules.ManageSiteCustomThemesModule.listeners.saveTheme(event)"/>
	</div>
</form>

<h2>Notes:</h2>
<ul>
	<li>
		The import feature will look for the first occurence of <tt>[[code]]...[[/code]]</tt> block
		(or <tt>[[code type="css"]]...[[/code]]</tt>) and will copy its content.
	</li>
	<li>
		The theme will <u>not</u> be updated automatically when you change the wiki
		page with the CSS block. You must sync it here.
	</li>
	<li>
		If you make changes that apply to live wiki pages <u>make sure the changes
		do not affect site functionality</u>. Please check your styles with different
		browsers and create a sandbox category with test pages for experiments.
	</li>
</ul>


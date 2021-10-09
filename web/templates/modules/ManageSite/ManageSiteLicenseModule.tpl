<h1>{t}License{/t}</h1>

<p>
	It is <u>very</u> important to clarify the copyright and ownership issues for your site.
	We highly recommend setting an open "<a href="https://en.wikipedia.org/wiki/Copyleft"
	target="_blank">copyleft</a>" license that allows making the
	Content more or less free to copy, modify and use.
</p>
<p>
	This is particularly important when your Site is created and edited collaboratively.
</p>
<p>
	Read more about <a href="https://creativecommons.org/about/licenses/meet-the-licenses"
	target="_blank">Creative Commons licenses</a>, use a <a href="https://creativecommons.org/license/"
	target="_blank">wizard</a> to select the proper license or just visit
	<a href="https://creativecommons.org/" target="_blank">Creative Commons</a>.
</p>

<div>
	<table class="form">
		<tr>
			<td>
				{t}Choose the category{/t}:
			</td>
			<td>
				<select name="category" size="15" id="sm-license-cats">
					{foreach from=$categories item=category}
						<option value="{$category->getCategoryId()}" style="padding: 0 1em" {if $category->getName()=="_default"}selected="selected"{/if}>{$category->getName()|escape}</option>
					{/foreach}
				</select>
			</td>
			<td>
				<div id="sm-license-noind">
					{t}Inherit from <tt>_default</tt>{/t}: <input class="checkbox" type="checkbox" id="sm-license-noin"/>
				</div>
			</td>
		</tr>
	</table>
	<div id="sm-license-list">
		<table class="form">
			<tr>
				<td>
					{t}Choose the license{/t}:
				</td>
				<td>

					<select id="sm-license-lic">
						{foreach from=$licenses item=license}
							<option value="{$license->getLicenseId()}">{$license->getName()|escape}</option>
						{/foreach}
					</select>
				</td>
			</tr>
		</table>
	</div>
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" id="sm-license-cancel"/>
		<input type="button" value="{t}save changes{/t}" id="sm-license-save"/>
	</div>

</div>

<div id="sm-license-preview" style="margin-bottom:2em">
	<h2>{t}License preview{/t}:</h2>
	{foreach from=$licenses item=license}
		<div id="sm-prev-license-{$license->getLicenseId()}" class="license-area">
			{ltext lang="en"}
				{$license->getDescription()|replace:'%%UNLESS%%':'Unless stated otherwise Content of this page is licensed under'}
			{/ltext}
			{ltext lang="pl"}
				{$license->getDescription()|replace:'%%UNLESS%%':'Jeśli nie zaznaczono inaczej, Zawartość tej strony dostępna jest na licencji'}
			{/ltext}
		</div>
	{/foreach}
</div>

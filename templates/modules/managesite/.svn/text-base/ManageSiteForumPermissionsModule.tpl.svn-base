<h1>{t}Forum permissions{/t}</h1>


<div style="margin-bottom: 1em">
	<table class="form">
		<tr>
			<td>
				{t}Forum category{/t}:
			</td>
			<td>
				<select name="category" size="10" id="sm-perms-cats">
					<option selected="selected" value="" style="margin: 0.5em 0">{t}default forum permissions{/t}</option>
					{foreach from=$groups item=group}
						<optgroup  label="{$group->getName()|escape}">
							{assign var=groupId value=$group->getGroupId()}
							{foreach from=$categories[$groupId] item=category}
								<option value="{$category->getCategoryId()}" style="padding: 0 1em" >{$category->getName()|escape}</option>
							{/foreach}
						</optgroup>
					{/foreach}
				</select>
			</td>
			<td>
				<div id="sm-perms-noind">
					{t}Use default permissions{/t} <input class="checkbox" type="checkbox" id="sm-perms-noin"/>
				</div>
			</td>
		</tr>
	</table>
</div>

<div id="sm-perms-table">
	<form id="sm-perms-form">
		<table class="form grid">
			<tr>
				<td>
					&nbsp;
				</td>
				<td>
					{t}anonymous{/t}
				</td>
				<td>
					{t}registered{/t}
				</td>
				<td>
					{t}site member{/t}
				</td>
				<td>
					{t}author{/t}
				</td>
			</tr>
			<tr>
				<td>
					{t}add new posts{/t}
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="p-a" id="sm-p-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="p-r" id="sm-p-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="p-m" id="sm-p-m"/>
				</td>
				<td>
					&nbsp;
				</td>
				
			</tr>
			<tr>
				<td>
					{t}create new threads{/t}
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="t-a" id="sm-t-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="t-r" id="sm-t-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="t-m" id="sm-t-m"/>
				</td>
				<td>
					&nbsp;
				</td>
				
			</tr>
			<tr>
				<td>
					{t}edit posts (and threads meta){/t}
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="e-a" id="sm-e-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="e-r" id="sm-e-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="e-m" id="sm-e-m"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="e-o" id="sm-e-o"/>
				</td>
				
			</tr>
			{*<tr>
				<td>
					split threads 
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="s-a" id="sm-s-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="s-r" id="sm-s-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="s-m" id="sm-s-m"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="s-o" id="sm-s-o"/>
				</td>
				
			</tr>*}
			
		</table>
		
	</form>
</div>
<form>
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" id="sm-perms-cancel"/>
		<input type="button" value="{t}save changes{/t}" id="sm-perms-save"/>
	</div>
</form>
<input type="hidden" id="default-forum-permissions" value="{$defaultPermissions}"/>



<h2>{t}A few notes{/t}</h2>
<p>
	{t}<strong>Site administrators</strong> and <strong>forum moderators</strong> are not mentioned in the table above because they
	automatically have all the rights within the site.{/t}
</p>

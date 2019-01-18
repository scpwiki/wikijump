<h1>{t}Permissions{/t}</h1>

<p>
	{t}Permissions are one of the most important settings for your Site. You can determine 
	who can edit contents of your pages and alter the structure of the Site.{/t}
</p>
<p>
	{t}It is up to you to choose who can modify the contents of the Site: 
	anyone, selected people only or just you as the maintainer of the Site.{/t}
</p>

<div>
	<table class="form">
		<tr>
			<td>
				{t}Choose the category{/t}:
			</td>
			<td>
				<select name="category" size="10" id="sm-perms-cats">
					{foreach from=$categories item=category}
						<option value="{$category->getCategoryId()}" style="padding: 0 1em" {if $category->getName()=="_default"}selected="selected"{/if}>{$category->getName()|escape}</option>
					{/foreach}
				</select>
			</td>
			<td>
				<div id="sm-perms-noind">
					{t}Inherit from <tt>_default</tt>{/t}: <input class="checkbox" type="checkbox" id="sm-perms-noin"/>
				</div>
			</td>
		</tr>
	</table>
</div>

<div id="sm-perms-table">
	<form id="sm-perms-form">
		<table class="form grid" style="font-size: 90%;">
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
					{t}page creator/owner{/t}
				</td>
			</tr>
			{*
			<tr>
				<td>
					view 
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="v-a" id="sm-v-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="v-r" id="sm-v-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="v-m" id="sm-v-m"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="v-o" id="sm-v-o"/>
				</td>
				
			</tr>
			*}
			<tr>
				<td>
					{t}edit{/t}
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
			<tr>
				<td>
					{t}create pages{/t}
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="c-a" id="sm-c-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="c-r" id="sm-c-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="c-m" id="sm-c-m"/>
				</td>
				<td>
					&nbsp; {*<input class="checkbox" type="checkbox" name="c-o"  id=/>*}
				</td>
				
			</tr>
			<tr>
				<td>
					{t}move/rename pages{/t}
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="m-a" id="sm-m-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="m-r" id="sm-m-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="m-m" id="sm-m-m"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="m-o" id="sm-m-o"/>
				</td>
				
			</tr>
			<tr>
				<td>
					{t}delete pages{/t}
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="d-a" id="sm-d-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="d-r" id="sm-d-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="d-m" id="sm-d-m"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="d-o" id="sm-d-o"/>
				</td>
				
			</tr>
			<tr>
				<td>
					{t}upload files{/t}
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="a-a" id="sm-a-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="a-r" id="sm-a-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="a-m" id="sm-a-m"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="a-o" id="sm-a-o"/>
				</td>
				
			</tr>
			<tr>
				<td>
					{t}rename files{/t}
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="r-a" id="sm-r-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="r-r" id="sm-r-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="r-m" id="sm-r-m"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="r-o" id="sm-r-o"/>
				</td>
				
			</tr>
			<tr>
				<td>
					{t}replace/move/delete files{/t}
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="z-a" id="sm-z-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="z-r" id="sm-z-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="z-m" id="sm-z-m"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="z-o" id="sm-z-o"/>
				</td>
				
			</tr>
			<tr>
				<td>
					{t}show page options to{/t}
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="o-a" id="sm-o-a"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="o-r" id="sm-o-r"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="o-m" id="sm-o-m"/>
				</td>
				<td>
					<input class="checkbox" type="checkbox" name="o-o" id="sm-o-o"/>
				</td>
				
			</tr>
		</table>
		
	</form>
</div>
<form>
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" id="sm-perms-cancel"/>
		<input type="button" value="{t}save changes{/t}" id="sm-perms-save"/>
	</div>
</form>
<div>
<h2>{t}A few notes{/t}</h2>
<p>
	{t}<strong>Site administrators</strong> and <strong>page moderators</strong> are not mentioned in the table above because they
	automatically have all the rights within the site.{/t}
</p>
<p>
	{t}"Page creators/owners" modifier can add extra control and and is suplementary to other options. 
	E.g. it is possible to deny edit permissions in general but allow only to the author that
	has created the page.{/t}
</p>
</div>
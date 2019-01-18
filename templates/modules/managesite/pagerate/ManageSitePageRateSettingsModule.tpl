<h1>{t}Page rating{/t}</h1>

<p>
	{t}Page ratings are not only a way to enable Users to vote on high/low quality content
	but also can be used to organize contests, competitions etc. 
	Anyway - the feature has to be enabled first.{/t}
</p>
<p>
	{t}This is a "per category" setting which means you can have rating enabled only for a 
	subclass of your Site. If a category setting reads "default" it means a value from the 
	"_default" category is being inherited.{/t}
</p>



<form id="sm-pr-form">
	<div id="sm-pr-categories">
		<table class="form grid" style="font-size: 85%">
			<tr>
				<th>
					{t}category{/t}
				</th>
				<th>
					{t}enable?{/t}
				</th>
				<th>
					{t}who can rate?{/t}
				</th>
				<th>
					{t}anonymously?{/t}
				</th>
				<th>
					{t}type{/t}
				</th>
			</tr>
			{foreach from=$categories item=category}
				<tr>
					<td>
						{$category->getName()|escape}
					</td>
					<td>						
						<select id="cat235-{$category->getCategoryId()}-e" onchange="WIKIDOT.modules.ManageSitePageRateSettingsModule.utils.updateVis({$category->getCategoryId()})">
							{if $category->getName() != '_default'}
								<option value="default"  {if $category->getRatingEnabled() === null}selected="selected"{/if}>{t}default{/t}</option>
							{/if}
							<option value="disabled" {if $category->getRatingEnabled() === false}selected="selected"{/if}>disabled</option>
							<option value="enabled" {if $category->getRatingEnabled() === true}selected="selected"{/if}>enabled</option>
						</select>
					</td>
					<td>
						<select id="cat235-{$category->getCategoryId()}-w">
							<option value="r" {if $category->getRatingBy() === "r"}selected="selected"{/if}>all registered Users</option>
							<option value="m" {if $category->getRatingBy() === "m"}selected="selected"{/if}>Site Members</option>
						</select>
					</td>
					<td>
						<select id="cat235-{$category->getCategoryId()}-v">
							<option value="a" {if $category->getRatingVisible() === "a"}selected="selected"{/if}>anonymously</option>
							<option value="v" {if $category->getRatingVisible() === "v"}selected="selected"{/if}>visible</option>
						</select>
					</td>
					<td>
						<select id="cat235-{$category->getCategoryId()}-t">
							<option value="P" {if $category->getRatingType() === "P"}selected="selected"{/if}>+ only</option>
							<option value="M" {if $category->getRatingType() === "M"}selected="selected"{/if}>+/-</option>
							{*<option value="S" {if $category->getRatingType() === "S"}selected="selected"{/if}>stars</option>*}
							
						</select>
					</td>
				</tr>
			{/foreach}
		</table>
	</div>
	
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" onclick="WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-welcome')"/>
		<input type="button" value="{t}save{/t}" onclick="WIKIDOT.modules.ManageSitePageRateSettingsModule.listeners.save(event)"/>
	</div>
</form>
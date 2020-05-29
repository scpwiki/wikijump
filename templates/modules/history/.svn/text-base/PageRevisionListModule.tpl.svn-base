{pager jsfunction="updatePagedList" total=$pagerData.total_pages known=$pagerData.known_pages current=$pagerData.current_page}

<table class="page-history">
	<tr>
		<td>
			{t}rev.{/t}
		</td>
		<td>
			&nbsp;
		</td>
		<td>
			{t}flags{/t}
		</td>
		<td>
			{t}actions{/t}
		</td>
		<td>
			{t}by{/t}
		</td>
		<td>
			{t}date{/t}
		</td>
		<td>
			{t}comments{/t}
		</td>
	</tr>
	{assign var=count value=0}
	{foreach from=$revisions item=pr}
	
	<tr id="revision-row-{$pr->getRevisionId()}">
		<td>{$pr->getRevisionNumber()}.</td>
		<td style="width: 5em" >
			<input id="{$pr->getRevisionId()}" type="radio" name="from" value="{$pr->getRevisionId()}" {if $count==1}checked="checked" {assign var=count value=2}{/if} />
			<input id="{$pr->getRevisionId()}" type="radio" name="to" value="{$pr->getRevisionId()}" {if $count==0}checked="checked" {assign var=count value=1}{/if} />
		</td>
		<td>
			{if $pr->getFlagNew()}
		 		<span class="spantip" title="{t}new page created{/t}">N</span>
		 	{/if}
		 	{if $pr->getFlagText()}
		 		<span class="spantip" title="{t}content source text changed{/t}">S</span>
		 	{/if}
		 	{if $pr->getFlagTitle()}
		 		<span class="spantip" title="{t}title changed{/t}">T</span>
		 	{/if}
		 	{if $pr->getFlagRename()}
		 		<span class="spantip" title="{t}page renamed/moved{/t}">R</span>
		 	{/if}  
		 	{if $pr->getFlagFile()}
		 		<span class="spantip" title="{t}file/attachment action{/t}">F</span>
		 	{/if}  
		 	{if $pr->getFlagMeta()}
		 		<span class="spantip" title="{t}meta data changed{/t}">M</span>
		 	{/if} 
		</td>
		<td style="width: 5em" class="optionstd">
			 <a title="{t}view page revision{/t}" href="javascript:;" onclick="showVersion({$pr->getRevisionId()})">V</a>
			 <a title="{t}view source of the revision{/t}" href="javascript:;" onclick="showSource({$pr->getRevisionId()})">S</a>
			 {if ($pr->getFlagNew() || $pr->getFlagText() || $pr->getFlagTitle()) && $page->getRevisionId() != $pr->getRevisionId() } {*&& $currentRevision->getSourceId() != $pr->getSourceId()}*}
			 	<a title="{t}revert to revision{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageHistoryModule.listeners.revert(event,{$pr->getRevisionId()})">R</a>
			 {/if}
		</td>
		<td style="width: 15em">{printuser user=$pr->getUserOrString() image="true"}</td>
		<td style="padding: 0 0.5em; width: 7em;"><span class="odate">{$pr->getDateLastEdited()->getTimestamp()}|%e %b %Y|agohover</span></td>
		<td style="font-size: 90%">{$pr->getComments()|escape}</td>
	</tr>
	{/foreach}
</table>
<script type="text/javascript" src="/common--javascript/yahooui/animation-min.js"></script>
{strip}<div class="simpletodo-box" id="simpletodo_{$listCounter}">
	<div class="title">{if $title}{$title|escape}{else}Here is a place for your title{/if}
	</div>
	<table class="simpletodo-format-table">
		<tr>
			<td>
				<div class="simpletodo-sub-box" id="simpletodo_sub_{$listCounter}">
					{if !$data}
						<div class="task">
							<span class="checkbox">
								<input type="checkbox" class="checkbox"/>
							</span>
							<span>
								<span class="text">Click me to edit !</span>
							</span>
							<span class="follow-link">
								<a href="javascript:;" class="icon1"><span>Follow link</span></a>
							</span>
							<span class="options">
								{if $canEdit}
									<a href="javascript:;" class="icon3"><span>Edit Link</span></a>
									<a href="javascript:;" class="icon2"><span>Remove</span></a>
								{/if}				
							</span>
						</div>
						<div class="task">
							<span class="checkbox">
								<input type="checkbox" class="checkbox"/>
							</span>
							<span>
								<span class="text">Drag me !</span>
							</span>
							<span class="follow-link">
								<a href="javascript:;" class="icon1">Follow Link</a>
							</span>
							<span class="options">
								{if $canEdit}
									<a href="javascript:;" class="icon3"><span>Edit Link</span></a>	
									<a href="javascript:;" class="icon2"><span>Remove</span></a>
								{/if}	
							</span>
						</div>	
					{else}
						{foreach from=$data item=itemData}
						<div class="task">
							<span class="checkbox">				
								<input type="checkbox" class="checkbox" {if $itemData->checked}checked="checked"{/if} {if !$canEdit}disabled="disabled"{/if}/>	
							</span>
							<span>
								<span class="text">{$itemData->text|escape}</span>
							</span>
							{if $itemData->link}
							<span class="follow-link proper-link">
								<a href="{$itemData->link|escape}" class="icon1"><span>Follow link</span></a>
							</span>
							{else}
							<span class="follow-link">
								<a href="javascript:;" class="icon1"><span>Follow link</span></a>
							</span>
							{/if}
							<span class="options">
								{if $canEdit}
									<a href="javascript:;" class="icon3"><span>Edit Link</span></a>
									<a href="javascript:;" class="icon2"><span>Remove</span></a>
								{/if}
							</span>
						</div>	
						{/foreach}
					{/if}		
				</div>
			</td>
		</tr>
	</table>
	<div class="bottom-options">
		{if $canEdit}	
			<a href="javascript:;" id="simpletodo-add-task-anchor{$listCounter}" onclick="WIKIDOT.modules.SimpleToDoModule.listeners.clickToAddTask(event,'simpletodo_{$listCounter}')"><img src="/common--images/todo/list-add.png"/>Add Item</a>
		{/if}
	</div>
	<div class="label">{$label}</div>
</div>{/strip}
{if $listCounter == 0}
<div id="simpletodo-data">
<span id="simpletodo-data-title">Here is a place for your title</span>
<span id="simpletodo-data-itemtext">Click me to edit !</span>
<span id="simpletodo-data-edit-permission">{if $canEdit}true{else}false{/if}</span>
</div>
{/if}
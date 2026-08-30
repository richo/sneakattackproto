default: update_rallies html/timecomp.html

deploy:
	cargo run --release --bin check
	fly deploy

update_rallies:
	curl https://sneakattackrally.com/ARACombinerThing/data/nonARA/allRallies.json | jq > nonARArallies.json
	curl https://sneakattackrally.com/ARACombinerThing/data/2024rallies.json | jq > 2024rallies.json
	curl https://sneakattackrally.com/ARACombinerThing/data/2025rallies.json | jq > 2025rallies.json
	curl https://sneakattackrally.com/ARACombinerThing/data/2026rallies.json | jq > 2026rallies.json
	curl https://sneakattackrally.com/ARACombinerThing/data/uidsSmall.json | jq > uidsSmall.json

html/timecomp.html: html/timecomp.html.erb html/generate-form.rb 2024rallies.json 2025rallies.json
	cd html && bundle exec ruby generate-form.rb

copy: html/timecomp.html
	cp html/timecomp.html ~/src/recce.tools/


.PHONY: default update_rallies copy deploy

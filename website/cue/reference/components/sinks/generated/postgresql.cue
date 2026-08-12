package metadata

generated: components: sinks: postgresql: configuration: {
	acknowledgements: {
		description: """
			Controls how acknowledgements are handled for this sink.

			See [End-to-end Acknowledgements][e2e_acks] for more information on how event acknowledgement is handled.

			[e2e_acks]: https://vector.dev/docs/architecture/end-to-end-acknowledgements/
			"""
		required: false
		type: object: options: enabled: {
			description: """
				Controls whether or not end-to-end acknowledgements are enabled.

				When enabled for a sink, any source that supports end-to-end
				acknowledgements that is connected to that sink waits for events
				to be acknowledged by **all connected sinks** before acknowledging them at the source.

				Enabling or disabling acknowledgements at the sink level takes precedence over any global
				[`acknowledgements`][global_acks] configuration.

				[global_acks]: https://vector.dev/docs/reference/configuration/global-options/#acknowledgements
				"""
			required: false
			type: bool: {}
		}
	}
	conflicts: {
		description: "Supported options to deal with insert conflicts."
		required:    false
		type: object: options: {
			action: {
				description: "The action to take on insert conflicts."
				required:    true
				type: string: enum: {
					nothing: "Drop conflicting insert values without generating an error."
					update:  "Update fields of the existing row if the insert causes a conflict."
				}
			}
			fields: {
				description: """
					The list of fields that should be updated with event object. These fields
					need to be defined in the schema configuration section.
					"""
				relevant_when: "action = \"update\""
				required:      true
				type: array: items: type: string: {}
			}
			target: {
				description: "The list of unique constrained fields that would cause a conflict."
				required:    true
				type: array: items: type: string: {}
			}
		}
	}
	connection: {
		description: """
			The connection URI for the postgres database to write data into. This is of the form
			`postgresql://[userspec@][hostspec][/dbname]`
			   where userspec is `user[:password]`
			   and hostspec is `[host][:port]`
			"""
		required: true
		type: string: {}
	}
	max_pool_size: {
		description: """
			Maximum size of the Postgres connection pool for this instance.
			Defaults to 4
			"""
		required: false
		type: uint: default: 4
	}
	schema: {
		description: "Schema information for the output table in PostgreSQL."
		required:    true
		type: object: options: {
			fields: {
				description: "Column/Event field mapping."
				required:    false
				type: array: {
					default: []
					items: type: object: options: {
						name: {
							description: "The name of the table column to write the data into."
							required:    true
							type: string: {}
						}
						path: {
							description: "The VRL path used to access the data from the event object."
							required:    true
							type: string: {}
						}
					}
				}
			}
			table: {
				description: "Name of the table to write information to."
				required:    true
				type: string: {}
			}
		}
	}
}

'use strict'
const command='sudo docker run --rm -v "$PWD":/var/task -i -e DOCKER_LAMBDA_USE_STDIN=1 lambci/lambda:provided -bootstrap ./target/lambda/density/bootstrap'
const {exec} = require('child_process')
const spawnCommand=(jsonFile, callback)=>{
    exec('cat '+jsonFile+' | '+command, callback)
}
jest.setTimeout(20000)
describe('density', ()=>{
    it('returns array of value and points', done=>{
        spawnCommand('./tests/parameter3.json', (err, result)=>{
            if(err){
                throw(err)
            }
            const res=JSON.parse(JSON.parse(result).body)
            expect(Array.isArray(res))
            expect(res[0].value).toBeDefined()
            expect(res[0].at_point).toBeDefined()
            done()
        })
    })
})
import {
    LOGOUT,
    UPDATE_API_KEY,
    UPDATE_AWS_CLIENT, 
    LOGIN_ERROR,
    REPEAT_VISITOR
} from '../actions/constants'
import queryString from 'query-string'
const {token, usagePlanId}=queryString.parse(location.search)
const defaultQuery={
    repeatVisitor:false
}
const stateWithQuery={
    ...defaultQuery,
    token,
    usagePlanId
}
export default (state=stateWithQuery, action)=>{
    switch(action.type){
        case UPDATE_API_KEY:
            return {
                ...state,
                apiKey:action.value
            }
        case UPDATE_AWS_CLIENT:
            return {
                ...state, 
                isSignedIn:true, 
                cognitoUser:action.user, 
                error:null
            }
        case REPEAT_VISITOR:
            return {
                ...state,
                repeatVisitor:true
            }
        case LOGIN_ERROR:
            return {...state, error:action.value}
        case LOGOUT:
            return defaultQuery
        default:
            return state
    }
}